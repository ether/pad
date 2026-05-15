pub struct ParsedPadUrl {
    pub remote_base: String,
    pub pad_id: String,
}

pub fn parse_pad_url(s: &str) -> anyhow::Result<ParsedPadUrl> {
    let url = url::Url::parse(s).map_err(|e| anyhow::anyhow!("parse url: {e}"))?;
    if url.scheme() != "http" && url.scheme() != "https" {
        anyhow::bail!("unsupported scheme: {}", url.scheme());
    }
    let path = url.path();
    let Some(rest) = path.strip_prefix("/p/") else {
        anyhow::bail!("URL path doesn't start with /p/");
    };
    let pad_id = rest.split('/').next().unwrap_or("").to_string();
    if pad_id.is_empty() {
        anyhow::bail!("pad id is empty");
    }
    let host = url.host_str().ok_or_else(|| anyhow::anyhow!("no host"))?;
    let port_part = match url.port() {
        Some(p) => format!(":{p}"),
        _ => String::new(),
    };
    let remote_base = format!("{}://{}{}", url.scheme(), host, port_part);
    Ok(ParsedPadUrl {
        remote_base,
        pad_id,
    })
}
