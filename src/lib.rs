use anyhow::{Context, Result, anyhow, bail};
use itertools::Itertools;

mod input;
mod output;
use input as i;
use output as o;

fn find_name_match(base: &i::Base, module: &i::Module, name: &str) -> Result<u64> {
    for id in module.items.iter() {
        let child_val_name = base.index[&id]
            .name
            .as_ref()
            .ok_or(anyhow!("referenced element has no name"))?;

        if child_val_name == name {
            return Ok(*id);
        }
    }

    Err(anyhow!("element {} does not exist", name))
}

fn find<'a>(base: &i::Base, mut path: impl Iterator<Item = &'a str>) -> Result<u64> {
    let mut module_id = base.root;

    loop {
        let Some(next_name) = path.next() else {
            return Ok(module_id);
        };

        let element = &base.index[&module_id];
        let i::InnerElement::Module(module) = &element.inner else {
            bail!("element {:?} was not a module", element.name)
        };

        module_id = find_name_match(base, module, next_name).context("could not find element")?;
    }
}

fn build_streams_from_parsed_defs(
    meta: &o::Metadata,
    parsed_defs: Vec<i::ParsedDef>,
) -> Result<Vec<o::Stream>> {
    parsed_defs
        .into_iter()
        .map(|i::ParsedDef { label, address, ty }| -> Result<_> {
            let subscription = o::Subscription {
                subject: address.clone(),
            };
            let o::TypeMetadata {
                telemetry_type,
                vector_keys,
                flag_names,
            } = meta
                .types
                .get(&ty)
                .with_context(|| format!("No telemetry type registered for {}", ty))?
                .clone();
            let parser_id = meta
                .parsers
                .get(&telemetry_type)
                .with_context(|| format!("No parser registered for {}", telemetry_type))?
                .clone();
            Ok(o::Stream {
                address,
                label,
                subscription,
                telemetry_type,
                vector_keys,
                flag_names,
                parser_id,
                publishable: false,
                format: meta.stream_metadata.format.clone(),
                provider_id: meta.provider.id.clone(),
                capacity: meta.stream_metadata.capacity,
            })
        })
        .collect()
}

fn build_streams_from_def(meta: &o::Metadata, def: i::Def) -> Result<Vec<o::Stream>> {
    let address: String = def.address;
    let label: String = def.label;
    let subscription = o::Subscription {
        subject: address.clone(),
    };
    let publishable = def.command;

    let o::TypeMetadata {
        telemetry_type,
        vector_keys,
        flag_names,
    } = meta
        .types
        .get(&def.ty)
        .with_context(|| format!("No telemetry type registered for {}", def.ty))?
        .clone();
    let parser_id = meta
        .parsers
        .get(&telemetry_type)
        .with_context(|| format!("No parser registered for {}", telemetry_type))?
        .clone();
    Ok(vec![o::Stream {
        address,
        label,
        subscription,
        telemetry_type,
        vector_keys,
        flag_names,
        parser_id,
        publishable,
        format: meta.stream_metadata.format.clone(),
        provider_id: meta.provider.id.clone(),
        capacity: meta.stream_metadata.capacity,
    }])
}

fn build_streams_from_docs(meta: &o::Metadata, docs: &str) -> Result<Vec<o::Stream>> {
    let tooling_block: String = pulldown_cmark::Parser::new(docs)
        .skip_while(|ev| {
            !matches!(
                ev,
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::CodeBlock(
                    pulldown_cmark::CodeBlockKind::Fenced(pulldown_cmark::CowStr::Borrowed(
                        "tooling"
                    ))
                ))
            )
        })
        .skip(1)
        .take_while(|ev| {
            !matches!(
                ev,
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::CodeBlock)
            )
        })
        .filter_map(|ev| {
            if let pulldown_cmark::Event::Text(text) = ev {
                Some(text.to_string())
            } else {
                None
            }
        })
        .collect();

    let Ok(chell_def): Result<i::Def, _> = toml::from_str(&tooling_block) else {
        bail!("Could not parse tooling block")
    };

    if let Some(parsed_defs) = chell_def.parsed {
        build_streams_from_parsed_defs(meta, parsed_defs)
    } else {
        build_streams_from_def(meta, chell_def)
    }
}

fn generate_streams_recursive(
    meta: &o::Metadata,
    base: &i::Base,
    entry_id: u64,
) -> Result<Vec<o::Stream>> {
    let element = &base.index[&entry_id];

    match &element.inner {
        i::InnerElement::Module(m) => m
            .items
            .iter()
            .map(|id| generate_streams_recursive(meta, base, *id))
            .flatten_ok()
            .collect(),
        i::InnerElement::Struct(_) => {
            let Some(docs) = element.docs.as_ref() else {
                return Ok(vec![]);
            };
            build_streams_from_docs(meta, docs)
        }
        _ => Ok(vec![]),
    }
}

pub fn generate_tilestion_tm(
    meta: o::Metadata,
    rustdoc_json: String,
    def_paths: Vec<String>,
) -> Result<String> {
    let value: i::Base = serde_json::from_str(&rustdoc_json).context("could not parse json")?;

    let streams: Vec<_> = def_paths
        .iter()
        .map(|definitions_path| -> Result<_> {
            let path_parts = definitions_path.split("::");
            let element_id = find(&value, path_parts).context("could not find def path")?;

            generate_streams_recursive(&meta, &value, element_id)
        })
        .flatten_ok()
        .collect::<Result<_>>()?;

    let generated_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    const FORMAT_VERSION: u32 = 3;

    let out = o::Base {
        format_version: FORMAT_VERSION,
        version: meta.version,
        generated_at,
        source: meta.source,
        providers: vec![meta.provider],
        streams,
    };

    let out_json = serde_json::to_string(&out).context("could not create json")?;

    Ok(out_json)
}
