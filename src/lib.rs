use structopt::StructOpt;
use url::{Url, Host};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_test_fn() {
        println!("Henlo frens");
    }

    #[test]
    fn url_parse_test() {
        let git_url = Url::parse("https://github.com/rust-lang/rust").unwrap();
        assert!(git_url.host() == Some(Host::Domain("github.com")));
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "RePossess", about = "Convert git repos into encoded images")]
pub struct Cli {
    /// GitHub or GitLab repository URL
    #[structopt(short = "u", long)]
    pub url: String
}

pub mod filehandle {
    use super::*;

    pub fn download_file(url: &String) -> Result<(), &'static str> {
        let parsed_url: Url = match Url::parse(&url) {
            Ok(p) => {p},
            Err(_) => {return Err("Could not parse URL");}
        };
        println!("{:#?}", url);

        let host = parsed_url.host();
        let hostname = match host {
            Some(x) => {
                println!("{:?}", x);
                if x != Host::Domain("github.com") && x != Host::Domain("gitlab.com")  {
                    return Err("Please enter only GitHub or GitLab URLs");
                }
                x.to_string()
            },
            None => {
                return Err("Please enter a valid URL");
            }
        };

        Ok(())
    }
}