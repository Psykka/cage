pub const CONFIG_FILENAME: &str = ".cagerc.toml";

pub const DEFAULT_CAGERC: &str = r##"# cagerc.toml — cage sandbox config
#
# Deny-by-default: nothing is reachable or writable unless a profile opts in.

[cage]
network = "deny"          # global kill-switch — never loosen this here

[filesystem]
ro = ["/usr", "/bin", "/lib"]   # read-only system dirs
rw = ["."]                      # only the current project is writable

[env]
unset = ["SSH_AUTH_SOCK"]       # keep the ssh-agent out of the cage

[internal]
# Host services exposed inside the cage (as its own localhost).
# The loopback starts empty — nothing here is reachable unless listed.
expose = []
# expose = ["127.0.0.1:5432", "127.0.0.1:11434"]  # e.g. postgres, ollama


# ------------------------------------------------------------------
#  Profiles
#
#  A profile = which hosts it may reach (allow) + which secrets it
#  carries (pass, pulled from the host env). Anything omitted falls
#  back to the [filesystem]/[env] defaults above.
# ------------------------------------------------------------------

# AI agents
[profiles.claude]
allow = ["api.anthropic.com:443"]
pass  = ["ANTHROPIC_API_KEY"]

[profiles.codex]
allow = ["api.openai.com:443"]
pass  = ["OPENAI_API_KEY"]

[profiles.gemini]
allow = ["generativelanguage.googleapis.com:443"]
pass  = ["GEMINI_API_KEY"]

# Everything else — the engine cages any process, not just agents.
[profiles.ci]
# Run untrusted build/test with zero network — kills dependency exfiltration.
allow = []

[profiles.offline]
# Inspect a suspicious repo: no network, read-only, can't touch anything.
allow = []
rw    = []"##;
