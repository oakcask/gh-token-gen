fn main() -> Result<(), Box<dyn std::error::Error>> {
    wasm_actions_build::generate_recommended()?;
    add_permission_inputs()?;
    Ok(())
}

fn add_permission_inputs() -> Result<(), Box<dyn std::error::Error>> {
    const PERMISSIONS: &[&str] = &[
        "actions",
        "administration",
        "artifact-metadata",
        "attestations",
        "checks",
        "codespaces",
        "contents",
        "custom-properties-for-organizations",
        "dependabot-secrets",
        "deployments",
        "discussions",
        "email-addresses",
        "enterprise-custom-properties-for-organizations",
        "environments",
        "followers",
        "git-ssh-keys",
        "gpg-keys",
        "interaction-limits",
        "issues",
        "members",
        "merge-queues",
        "metadata",
        "organization-administration",
        "organization-announcement-banners",
        "organization-copilot-seat-management",
        "organization-custom-org-roles",
        "organization-custom-properties",
        "organization-custom-roles",
        "organization-events",
        "organization-hooks",
        "organization-packages",
        "organization-personal-access-token-requests",
        "organization-personal-access-tokens",
        "organization-plan",
        "organization-projects",
        "organization-secrets",
        "organization-self-hosted-runners",
        "organization-user-blocking",
        "packages",
        "pages",
        "profile",
        "pull-requests",
        "repository-custom-properties",
        "repository-hooks",
        "repository-projects",
        "secret-scanning-alerts",
        "secrets",
        "security-events",
        "single-file",
        "starring",
        "statuses",
        "team-discussions",
        "vulnerability-alerts",
        "workflows",
    ];

    let mut action = std::fs::read_to_string("action.yaml")?;
    if action.contains("  permission-actions:") {
        return Ok(());
    }

    let permission_inputs = PERMISSIONS
        .iter()
        .map(|permission| {
            format!(
                "  permission-{permission}:\n    default: \"\"\n    description: GitHub App permission level to grant to the access token\n"
            )
        })
        .collect::<String>();
    action = action.replace("\noutputs:\n", &format!("\n{permission_inputs}outputs:\n"));
    std::fs::write("action.yaml", action)?;

    Ok(())
}
