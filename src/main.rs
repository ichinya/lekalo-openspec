use serde_json::{json, Value};
use std::io::{self, Read};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    io::stdin()
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("request-limit".into());
    }
    let request: Value = serde_json::from_slice(&bytes)?;
    if request["protocol"] != "lekalo/runtime/v1"
        || !request["request_id"]
            .as_str()
            .is_some_and(|id| id.len() == 71 && id.starts_with("sha256:"))
    {
        return Err("request-envelope".into());
    }
    let result = dispatch(
        request["operation"].as_str().ok_or("operation")?,
        &request["input"],
    )?;
    print!(
        "{}",
        json!({"protocol":"lekalo/runtime/v1","request_id":request["request_id"],"result":result})
    );
    Ok(())
}

use lekalo_openspec::{EntryType, Loaded, ReadError, Source};
use std::cell::RefCell;
struct Replay<'a> {
    reads: &'a Value,
    pending: RefCell<Vec<Value>>,
}
impl Replay<'_> {
    fn lookup(&self, kind: &str, path: &str) -> Result<Value, ReadError> {
        let key = format!("{kind}:{path}");
        let Some(value) = self.reads.get(&key) else {
            self.pending
                .borrow_mut()
                .push(json!({"kind":kind,"path":path}));
            return Err(ReadError::Io);
        };
        if let Some(error) = value.get("error") {
            return Err(match error.as_str() {
                Some("not-found") => ReadError::NotFound,
                Some("limit") => ReadError::Limit,
                _ => ReadError::Io,
            });
        }
        Ok(value.clone())
    }
}
impl Source for Replay<'_> {
    fn entries(&self, path: &str) -> Result<Vec<(String, EntryType)>, ReadError> {
        self.lookup("entries", path)?["entries"]
            .as_array()
            .ok_or(ReadError::Io)?
            .iter()
            .map(|v| {
                Ok((
                    v[0].as_str().ok_or(ReadError::Io)?.to_owned(),
                    if v[1] == "directory" {
                        EntryType::Directory
                    } else {
                        EntryType::Other
                    },
                ))
            })
            .collect()
    }
    fn read_file_opt(
        &self,
        dir: &str,
        name: &str,
        max: usize,
    ) -> Result<Option<Vec<u8>>, ReadError> {
        let value = self.lookup("file", &format!("{dir}/{name}"))?;
        if value["content"].is_null() {
            return Ok(None);
        }
        let bytes = value["content"]
            .as_str()
            .ok_or(ReadError::Io)?
            .as_bytes()
            .to_vec();
        if bytes.len() > max {
            return Err(ReadError::Limit);
        }
        Ok(Some(bytes))
    }
}
fn dispatch(op: &str, input: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    if op == "describe" {
        return Ok(
            json!({"id":"lekalo-openspec","version":"0.1.0","rules_api":[],"capabilities":["requirements.snapshot/v1"],"operations":["requirements.snapshot"]}),
        );
    }
    if op != "requirements.snapshot" || input["schema"] != "lekalo/requirements-provider/v1" {
        return Err("operation-input".into());
    }
    let decl = lekalo_openspec::ProviderDecl {
        source: input["source"].as_str().ok_or("source")?.into(),
        root: input["root"].as_str().ok_or("root")?.into(),
    };
    let fs = Replay {
        reads: &input["reads"],
        pending: RefCell::new(Vec::new()),
    };
    let loaded = lekalo_openspec::load_snapshot(&fs, &decl);
    let requests = fs.pending.into_inner();
    if !requests.is_empty() {
        return Ok(
            json!({"schema":"lekalo/requirements-provider/v1","state":"read","requests":requests}),
        );
    }
    Ok(match loaded {
        Loaded::Absent => json!({"schema":"lekalo/requirements-provider/v1","state":"absent"}),
        Loaded::Invalid(detail) => {
            json!({"schema":"lekalo/requirements-provider/v1","state":"invalid","detail":detail})
        }
        Loaded::Resolved(s) => {
            json!({"schema":"lekalo/requirements-provider/v1","state":"complete","snapshot":{
                "source":s.source,"root":s.root,"status":"ok",
                "entries":s.entries.into_iter().map(|e|json!({"id":e.id,"digest":e.digest,"origin":if e.origin==lekalo_openspec::Origin::Accepted{"accepted"}else{"change"},"change":e.change,"path":e.path})).collect::<Vec<_>>(),
                "conflicts":s.conflicts.into_iter().map(|c|json!({"capability":c.capability,"id":lekalo_openspec::ids::requirement_id(&c.capability,&lekalo_openspec::ids::title_slug(&c.title).unwrap_or_default()),"detail":c.detail})).collect::<Vec<_>>(),
                "renames":s.renames,"removals":s.removals
            }})
        }
    })
}
