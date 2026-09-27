use base64::Engine;
use serde::Deserialize;
use whatsrook_sdk::{create_http_client, respond, respond_err, send_document, Request};

#[derive(Deserialize)]
struct RepoInfo {
    #[serde(default)]
    full_name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    stargazers_count: u64,
    #[serde(default)]
    forks_count: u64,
    #[serde(default)]
    open_issues_count: u64,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    default_branch: String,
    #[serde(default)]
    html_url: String,
}

#[derive(Deserialize)]
struct CommitItem {
    #[serde(default)]
    sha: String,
    #[serde(default)]
    commit: CommitDetail,
}

#[derive(Deserialize, Default)]
struct CommitDetail {
    #[serde(default)]
    message: String,
    #[serde(default)]
    author: Option<CommitAuthor>,
}

#[derive(Deserialize, Default)]
struct CommitAuthor {
    #[serde(default)]
    name: String,
    #[serde(default)]
    date: String,
}

#[derive(Deserialize)]
struct BranchItem {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct ReleaseItem {
    #[serde(default)]
    tag_name: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
}

#[derive(Deserialize)]
struct UserInfo {
    #[serde(default)]
    login: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    bio: Option<String>,
    #[serde(default)]
    public_repos: u64,
    #[serde(default)]
    followers: u64,
    #[serde(default)]
    following: u64,
    #[serde(default)]
    html_url: String,
}

#[derive(Deserialize)]
struct SearchResult {
    #[serde(default)]
    items: Vec<RepoInfo>,
}

fn parse_owner_repo(input: &str) -> Option<(String, String)> {
    let cleaned = input
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git")
        .trim_matches('/');

    let parts: Vec<&str> = cleaned.split('/').collect();
    if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

fn main() {
    let req = Request::load();
    let query = req.query();

    if query.is_empty() {
        let p = req.prefix();
        respond(format!(
            "*GitHub Plugin Usage:*\n\n\
             • `{p}git <owner/repo>` : Download repo as .zip archive\n\
             • `{p}git info <owner/repo>` : Repository metadata & metrics\n\
             • `{p}git commits <owner/repo>` : Recent commits\n\
             • `{p}git branches <owner/repo>` : List repository branches\n\
             • `{p}git releases <owner/repo>` : Release history & tags\n\
             • `{p}git user <username>` : User profile & statistics\n\
             • `{p}git search <query>` : Search GitHub repositories\n\n\
             *Example:* `{p}git Thruqe/whatsrook`"
        ));
        return;
    }

    let client = create_http_client(30);

    let sub = if !req.args.is_empty() {
        req.args[0].to_lowercase()
    } else {
        String::new()
    };

    match sub.as_str() {
        "info" => {
            if req.args.len() < 2 {
                respond_err(format!("Usage: {}git info <owner/repo>", req.prefix()));
            }
            let target = &req.args[1];
            if let Some((owner, repo)) = parse_owner_repo(target) {
                let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
                match client
                    .get(&url)
                    .header("Accept", "application/vnd.github.v3+json")
                    .send()
                {
                    Ok(resp) => {
                        if let Ok(info) = resp.json::<RepoInfo>() {
                            let mut text = format!(
                                "*GitHub: {}*\n\n\
                                 ⭐ *Stars:* {}\n\
                                 🍴 *Forks:* {}\n\
                                 ❗ *Open Issues:* {}\n\
                                 🌿 *Default Branch:* {}\n\
                                 💻 *Language:* {}\n\
                                 🔗 *URL:* {}",
                                info.full_name,
                                info.stargazers_count,
                                info.forks_count,
                                info.open_issues_count,
                                info.default_branch,
                                info.language.unwrap_or_else(|| "N/A".to_string()),
                                info.html_url
                            );
                            if let Some(desc) = info.description {
                                text.push_str(&format!("\n\n*Description:*\n{}", desc));
                            }
                            respond(text);
                            return;
                        }
                    }
                    Err(e) => respond_err(format!("Network error: {}", e)),
                }
            }
            respond_err(format!("Could not fetch repository info for `{}`", target));
        }

        "commits" => {
            if req.args.len() < 2 {
                respond_err(format!("Usage: {}git commits <owner/repo>", req.prefix()));
            }
            let target = &req.args[1];
            if let Some((owner, repo)) = parse_owner_repo(target) {
                let url = format!(
                    "https://api.github.com/repos/{}/{}/commits?per_page=5",
                    owner, repo
                );
                match client
                    .get(&url)
                    .header("Accept", "application/vnd.github.v3+json")
                    .send()
                {
                    Ok(resp) => {
                        if let Ok(commits) = resp.json::<Vec<CommitItem>>() {
                            if commits.is_empty() {
                                respond("No commits found.");
                                return;
                            }
                            let mut text = format!("*Recent Commits ({}/{}):*\n\n", owner, repo);
                            for c in commits {
                                let short_sha = if c.sha.len() > 7 { &c.sha[..7] } else { &c.sha };
                                let first_line =
                                    c.commit.message.lines().next().unwrap_or("").trim();
                                let author_info = c
                                    .commit
                                    .author
                                    .map(|a| {
                                        if !a.date.is_empty() {
                                            let date_part =
                                                a.date.split('T').next().unwrap_or(&a.date);
                                            format!("{} [{}]", a.name, date_part)
                                        } else {
                                            a.name
                                        }
                                    })
                                    .unwrap_or_else(|| "Unknown".to_string());
                                text.push_str(&format!(
                                    "• `{}` {} - _{}_\n",
                                    short_sha, first_line, author_info
                                ));
                            }
                            respond(text);
                            return;
                        }
                    }
                    Err(e) => respond_err(format!("Network error: {}", e)),
                }
            }
            respond_err(format!("Could not fetch commits for `{}`", target));
        }

        "branches" => {
            if req.args.len() < 2 {
                respond_err(format!("Usage: {}git branches <owner/repo>", req.prefix()));
            }
            let target = &req.args[1];
            if let Some((owner, repo)) = parse_owner_repo(target) {
                let url = format!(
                    "https://api.github.com/repos/{}/{}/branches?per_page=15",
                    owner, repo
                );
                match client
                    .get(&url)
                    .header("Accept", "application/vnd.github.v3+json")
                    .send()
                {
                    Ok(resp) => {
                        if let Ok(branches) = resp.json::<Vec<BranchItem>>() {
                            if branches.is_empty() {
                                respond("No branches found.");
                                return;
                            }
                            let list = branches
                                .iter()
                                .map(|b| format!("• {}", b.name))
                                .collect::<Vec<_>>()
                                .join("\n");
                            respond(format!("*Branches ({}/{}):*\n\n{}", owner, repo, list));
                            return;
                        }
                    }
                    Err(e) => respond_err(format!("Network error: {}", e)),
                }
            }
            respond_err(format!("Could not fetch branches for `{}`", target));
        }

        "releases" => {
            if req.args.len() < 2 {
                respond_err(format!("Usage: {}git releases <owner/repo>", req.prefix()));
            }
            let target = &req.args[1];
            if let Some((owner, repo)) = parse_owner_repo(target) {
                let url = format!(
                    "https://api.github.com/repos/{}/{}/releases?per_page=5",
                    owner, repo
                );
                match client
                    .get(&url)
                    .header("Accept", "application/vnd.github.v3+json")
                    .send()
                {
                    Ok(resp) => {
                        if let Ok(releases) = resp.json::<Vec<ReleaseItem>>() {
                            if releases.is_empty() {
                                respond("No releases found.");
                                return;
                            }
                            let mut text = format!("*Releases ({}/{}):*\n\n", owner, repo);
                            for r in releases {
                                let name = r.name.unwrap_or_else(|| r.tag_name.clone());
                                let date_str = r
                                    .published_at
                                    .as_deref()
                                    .and_then(|d| d.split('T').next())
                                    .unwrap_or("");
                                if !date_str.is_empty() {
                                    text.push_str(&format!(
                                        "• *{}* (`{}`) - _{}_\n",
                                        name, r.tag_name, date_str
                                    ));
                                } else {
                                    text.push_str(&format!("• *{}* (`{}`)\n", name, r.tag_name));
                                }
                            }
                            respond(text);
                            return;
                        }
                    }
                    Err(e) => respond_err(format!("Network error: {}", e)),
                }
            }
            respond_err(format!("Could not fetch releases for `{}`", target));
        }

        "user" => {
            if req.args.len() < 2 {
                respond_err(format!("Usage: {}git user <username>", req.prefix()));
            }
            let username = &req.args[1];
            let url = format!("https://api.github.com/users/{}", username);
            match client
                .get(&url)
                .header("Accept", "application/vnd.github.v3+json")
                .send()
            {
                Ok(resp) => {
                    if let Ok(u) = resp.json::<UserInfo>() {
                        let mut text = format!(
                            "*GitHub User: {}*\n\n\
                             👤 *Name:* {}\n\
                             📦 *Public Repos:* {}\n\
                             👥 *Followers:* {} | *Following:* {}\n\
                             🔗 *Profile:* {}",
                            u.login,
                            u.name.unwrap_or_else(|| "N/A".to_string()),
                            u.public_repos,
                            u.followers,
                            u.following,
                            u.html_url
                        );
                        if let Some(bio) = u.bio {
                            text.push_str(&format!("\n\n*Bio:*\n{}", bio));
                        }
                        respond(text);
                        return;
                    }
                }
                Err(e) => respond_err(format!("Network error: {}", e)),
            }
            respond_err(format!("Could not fetch user info for `{}`", username));
        }

        "search" => {
            if req.args.len() < 2 {
                respond_err(format!("Usage: {}git search <query>", req.prefix()));
            }
            let search_term = req.raw_args[req.args[0].len()..].trim();
            let encoded: String =
                url::form_urlencoded::byte_serialize(search_term.as_bytes()).collect();
            let url = format!(
                "https://api.github.com/search/repositories?q={}&per_page=5",
                encoded
            );
            match client
                .get(&url)
                .header("Accept", "application/vnd.github.v3+json")
                .send()
            {
                Ok(resp) => {
                    if let Ok(res) = resp.json::<SearchResult>() {
                        if res.items.is_empty() {
                            respond(format!("No repositories found for `{}`", search_term));
                            return;
                        }
                        let mut text =
                            format!("*GitHub Search Results for `{}`:*\n\n", search_term);
                        for item in res.items {
                            text.push_str(&format!(
                                "• *{}* (⭐ {})\n  _{}_\n  {}\n\n",
                                item.full_name,
                                item.stargazers_count,
                                item.description.unwrap_or_default(),
                                item.html_url
                            ));
                        }
                        respond(text.trim());
                        return;
                    }
                }
                Err(e) => respond_err(format!("Network error: {}", e)),
            }
            respond_err(format!("Search failed for `{}`", search_term));
        }

        _ => {
            // Default action: download repository archive (.zip)
            let target = if sub == "download" || sub == "clone" {
                if req.args.len() > 1 {
                    &req.args[1]
                } else {
                    ""
                }
            } else {
                &query
            };

            if let Some((owner, repo)) = parse_owner_repo(target) {
                let zip_url = format!("https://api.github.com/repos/{}/{}/zipball", owner, repo);
                match client
                    .get(&zip_url)
                    .header("Accept", "application/vnd.github.v3+json")
                    .send()
                {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            if let Ok(bytes) = resp.bytes() {
                                if !bytes.is_empty() {
                                    let b64 = format!(
                                        "data:application/zip;base64,{}",
                                        base64::engine::general_purpose::STANDARD.encode(&bytes)
                                    );
                                    let filename = format!("{}-{}.zip", owner, repo);
                                    send_document(
                                        &b64,
                                        &filename,
                                        Some(&format!("📦 Repository: {}/{}", owner, repo)),
                                    );
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => respond_err(format!("Network error downloading repo: {}", e)),
                }
            }

            respond_err(format!(
                "Invalid repository format. Please specify `<owner/repo>` (e.g. `{}git Thruqe/whatsrook`)",
                req.prefix()
            ));
        }
    }
}
