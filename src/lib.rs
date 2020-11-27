use structopt::StructOpt;
use url::{Host, Url};

#[cfg(test)]
mod tests {
    use filehandle::extract_repo_info;

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
}

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

pub mod filehandle {
    use super::*;

    pub fn extract_repo_from_cli(cli: &Cli) -> Result<Repo, &'static str> {
        let parsed_url: Url = match Url::parse(cli.url.trim_end_matches('/')) {
            Ok(p) => p,
            Err(_) => return Err("Could not parse URL"),
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
                    return Err("Please enter only GitHub or GitLab URLs");
                }
            },
            None => {
                return Err("Please enter a valid URL");
            }
        };

        Ok(given_repo)
    }

    pub fn extract_repo_info(parsed_url: &Url) -> Result<UserInfo, &'static str> {
        let url_path_segments = parsed_url
            .path()
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        if url_path_segments.len() < 2 {
            return Err("The URL does not seem to be a valid repo URL");
        }
        // println!("{:#?}", url_path_segments);

        Ok(UserInfo {
            user_name: String::from(*url_path_segments.get(0).unwrap()),
            repo_name: String::from(*url_path_segments.get(1).unwrap()),
        })
    }

    pub fn create_download_url(repo: &Repo) -> String {
        match repo.repo_type {
            RepoType::GitHub => {
                format!(
                    "{}/archive/{}.zip",
                    repo.url.clone().into_string(),
                    repo.branch
                )
            }
            RepoType::GitLab => {
                // https://gitlab.com/gitlab-org/gitaly/-/archive/master/gitaly-master.zip
                format!(
                    "{}/~/archive/{}/{}-{}.zip",
                    repo.url.clone().into_string(),
                    repo.branch,
                    repo.user_info.repo_name,
                    repo.branch
                )
            }
        }
    }

    pub fn download_repo(repo: &Repo) -> Result<(), &'static str> {
        let _download_url = create_download_url(&repo);
        // println!("{}", download_url);
        Ok(())
    }
}
