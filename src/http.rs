use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::error::Error;

const MAX_HEADER_SIZE: usize = 8192;

#[allow(dead_code)]
pub struct RequestInfo {
    pub method: String,
    pub path: String,
    pub host: Option<String>,
}

pub async fn parse_request(client: &mut TcpStream) -> Result<(RequestInfo, Vec<u8>), Error> {
    let mut buffer = vec![0u8; MAX_HEADER_SIZE];
    let mut filled = 0;

    loop {
        let n = client.read(&mut buffer[filled..]).await?;

        if n == 0 {
            return Err(Error::Other("client closed connection".into()));
        }

        filled += n;

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);

        match req.parse(&buffer[..filled])? {
            httparse::Status::Complete(_offset) => {
                let method = req.method.unwrap_or("").to_string();
                let path = req.path.unwrap_or("").to_string();

                let host = req
                    .headers
                    .iter()
                    .find(|h| h.name.eq_ignore_ascii_case("host"))
                    .and_then(|h| std::str::from_utf8(h.value).ok())
                    .map(|s| s.to_string());

                let info = RequestInfo { method, path, host };

                return Ok((info, buffer[..filled].to_vec()));
            }

            httparse::Status::Partial => {
                if filled >= MAX_HEADER_SIZE {
                    return Err(Error::Other("header too large".into()));
                }
                continue;
            }
        }
    }
}