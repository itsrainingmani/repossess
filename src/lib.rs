use std::error::Error;
use std::fmt::{self, Formatter};
use std::fs;
use structopt::StructOpt;
use url::{Host, Url};

#[derive(Debug, StructOpt)]
#[structopt(name = "RePossess", about = "Convert git repos into encoded images")]
pub struct Cli {
    /// GitHub or GitLab repository URL
    #[structopt(short = "u", long)]
    pub url: String,

    #[structopt(short = "b", long, default_value = "master")]
    pub branch: String,
}

#[derive(Debug, PartialEq)]
pub enum RepoType {
    GitHub,
    GitLab,
}

#[derive(Debug)]
pub struct Repo {
    pub url: Url,
    pub user_info: UserInfo,
    pub repo_type: RepoType,
    pub branch: String,
}

#[derive(Debug)]
pub struct UserInfo {
    pub user_name: String,
    pub repo_name: String,
}

macro_rules! simple_enum_error {
    ($($name: ident => $description: expr,)+) => {
        /// Errors that can occur during parsing.
        ///
        /// This may be extended in the future so exhaustive matching is
        /// discouraged with an unused variant.
        #[allow(clippy::manual_non_exhaustive)] // introduced in 1.40, MSRV is 1.36
        #[derive(PartialEq, Eq, Clone, Copy, Debug)]
        pub enum RepoError {
            $(
                $name,
            )+
            /// Unused variant enable non-exhaustive matching
            #[doc(hidden)]
            __FutureProof,
        }

        impl fmt::Display for RepoError {
            fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
                match *self {
                    $(
                        RepoError::$name => fmt.write_str($description),
                    )+
                    RepoError::__FutureProof => {
                        unreachable!("Don't abuse the FutureProof!");
                    }
                }
            }
        }
    }
}

impl Error for RepoError {}

simple_enum_error!(
    URLParseError => "Could not parse URL",
    NonGitURLError => "Please enter only GitHub or GitLab URLs",
    InvalidURLError => "Please enter a valid URL",
    InvalidRepoURLError => "The URL does not seem to be a valid repo URL",
    FileDownloadError => "The Git Repo could not be downloaded",
);

pub mod filehandle {
    use super::*;

    pub fn extract_repo_from_cli(cli: &Cli) -> Result<Repo, RepoError> {
        let parsed_url: Url = match Url::parse(cli.url.trim_end_matches('/')) {
            Ok(p) => p,
            Err(_) => return Err(RepoError::URLParseError),
        };
        // println!("{:#?}", url);

        let host = parsed_url.host();
        let given_repo: Repo = match host {
            Some(x) => match x {
                Host::Domain("github.com") => {
                    let user_info = extract_repo_info(&parsed_url)?;
                    Repo {
                        url: parsed_url,
                        user_info,
                        repo_type: RepoType::GitHub,
                        branch: cli.branch.clone(),
                    }
                }
                Host::Domain("gitlab.com") => {
                    let user_info = extract_repo_info(&parsed_url)?;
                    Repo {
                        url: parsed_url,
                        user_info,
                        repo_type: RepoType::GitLab,
                        branch: cli.branch.clone(),
                    }
                }
                _ => {
                    return Err(RepoError::NonGitURLError);
                }
            },
            None => {
                return Err(RepoError::InvalidURLError);
            }
        };

        Ok(given_repo)
    }

    fn extract_repo_info(parsed_url: &Url) -> Result<UserInfo, RepoError> {
        let url_path_segments = parsed_url
            .path()
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        if url_path_segments.len() < 2 {
            return Err(RepoError::InvalidRepoURLError);
        }
        // println!("{:#?}", url_path_segments);

        Ok(UserInfo {
            user_name: String::from(*url_path_segments.get(0).unwrap()),
            repo_name: String::from(*url_path_segments.get(1).unwrap()),
        })
    }

    fn create_download_url(repo: &Repo) -> String {
        match repo.repo_type {
            RepoType::GitHub => {
                format!(
                    "{}/archive/{}.zip",
                    repo.url.clone().into_string(),
                    repo.branch
                )
            }
            RepoType::GitLab => {
                // https://github.com/rust-lang/rust/archive/master.zip
                format!(
                    "{}/-/archive/{}/{}-{}.zip",
                    repo.url.clone().into_string(),
                    repo.branch,
                    repo.user_info.repo_name,
                    repo.branch
                )
            }
        }
    }

    fn download_repo(repo: &Repo) -> Result<(), Box<dyn Error>> {
        let download_url = create_download_url(&repo);
        println!("{}", download_url);
        let response = match reqwest::blocking::get(&download_url)?.error_for_status() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("ERROR {}", e.to_string());
                return Err(Box::new(e));
            }
        };

        let response_bytes = response.bytes()?;

        // Write to a file, if the file already exists, overwrite it
        fs::write("repo.zip", response_bytes)?;

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use filehandle::{create_download_url, download_repo, extract_repo_info};

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

        #[test]
        fn url_download_test() {
            let cli = Cli {
                url: String::from("https://github.com/rust-lang/rust"),
                branch: String::from("master"),
            };
            let git_repo = filehandle::extract_repo_from_cli(&cli).unwrap();
            println!("{:#?}", git_repo);
            assert_eq!(git_repo.repo_type, RepoType::GitHub);
        }

        #[test]
        fn url_malformed_git_url() {
            let git_url = "https://github.com/rust-lang/";
            let parsed_url: Url = Url::parse(git_url).unwrap();
            println!("{:#?}", extract_repo_info(&parsed_url));
        }

        #[test]
        fn download_url_github_test() {
            let cli = Cli {
                url: String::from("https://github.com/rust-lang/rust"),
                branch: String::from("master"),
            };
            let git_repo = filehandle::extract_repo_from_cli(&cli).unwrap();
            let download_url = create_download_url(&git_repo);

            assert_eq!(
                download_url,
                String::from("https://github.com/rust-lang/rust/archive/master.zip")
            )
        }

        #[test]
        fn download_url_gitlab_test() {
            let cli = Cli {
                url: String::from("https://gitlab.com/rust-lang/rust"),
                branch: String::from("master"),
            };
            let git_repo = filehandle::extract_repo_from_cli(&cli).unwrap();
            let download_url = create_download_url(&git_repo);

            assert_eq!(
                download_url,
                String::from("https://gitlab.com/rust-lang/rust/-/archive/master/rust-master.zip")
            )
        }

        #[test]
        fn file_download_test() {
            let cli = Cli {
                url: String::from("https://gitlab.com/rust-lang/rust"),
                branch: String::from("master"),
            };
            let git_repo = filehandle::extract_repo_from_cli(&cli).unwrap();
            download_repo(&git_repo).unwrap();

            assert_eq!(fs::metadata("repo.zip").unwrap().is_file(), true);
            // std::thread::sleep(std::time::Duration::from_secs(5));

            // Clean up download
            fs::remove_file("repo.zip").unwrap();
        }
    }
}
