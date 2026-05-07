use crate::pipeline::AgendaSummary;

pub fn batch_rule_prompt(agenda_description: &str, scope: Option<&str>) -> String {
    let scope_text = scope.unwrap_or("entire project");

    format!(
        r#"You are a security gatekeeper for a coding agent. Given the current project task, generate shell command patterns that should be pre-approved.

TASK: {agenda_description}
SCOPE: {scope_text}

Generate a JSON array of allow rules. Each rule has:
- command_pattern: glob for command name (e.g. "git", "npm", "cargo")
- args_pattern: glob for arguments (e.g. "status *", "install", "run *")  
- path_pattern: glob for working directory, or null for any
- notification: "silent" for trusted commands, "advisory" for edge cases
- reason: brief explanation
- confidence: 0.0 to 1.0

Be conservative. Only include commands clearly needed for this task.
Common safe commands: git status, git diff, git log, ls, cat, head, tail, find, grep.

Respond with ONLY the JSON array, no other text.

Example:
```json
[
  {{"command_pattern": "git", "args_pattern": "status *", "path_pattern": null, "notification": "silent", "reason": "read-only git operation", "confidence": 0.95}},
  {{"command_pattern": "npm", "args_pattern": "install", "path_pattern": null, "notification": "advisory", "reason": "dependency install may be needed", "confidence": 0.6}}
]
```

Now generate rules for the task above:"#
    )
}

pub fn inline_deliberation_prompt(
    command: &str,
    args: &[String],
    cwd: &str,
    agendas: &[AgendaSummary],
    recent_commands: &[String],
    warning_signs: &[String],
) -> String {
    let agenda_text = if agendas.is_empty() {
        "No active agendas.".to_string()
    } else {
        agendas
            .iter()
            .map(|a| {
                let scope = a.scope.as_deref().unwrap_or("unknown scope");
                format!("- [{}] {} (scope: {})", a.source, a.description, scope)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let history_text = if recent_commands.is_empty() {
        "No recent commands.".to_string()
    } else {
        recent_commands
            .iter()
            .take(20)
            .enumerate()
            .map(|(i, c)| format!("{}. {}", i + 1, c))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let warnings = warning_signs
        .iter()
        .map(|w| format!("- {}", w))
        .collect::<Vec<_>>()
        .join("\n");

    let args_str = args.join(" ");

    format!(
        r#"You are a security gatekeeper. Evaluate whether this command is consistent with the active project agenda.

COMMAND: {command} {args_str}
WORKING DIRECTORY: {cwd}

ACTIVE AGENDAS:
{agenda_text}

RECENT COMMAND HISTORY:
{history_text}

WARNING SIGNS TO WATCH FOR:
{warnings}

Respond in this EXACT format:
```
DECISION: ALLOW or BLOCK
CONFIDENCE: 0.0 to 1.0
REASON: one sentence explanation
```

Be conservative. If unsure, BLOCK. Only ALLOW if the command clearly fits the agenda."#
    )
}

pub fn question_generation_prompt(
    approved_command: &str,
    recent_commands: &[String],
) -> String {
    let history_text = recent_commands
        .iter()
        .take(10)
        .enumerate()
        .map(|(i, c)| format!("{}. {}", i + 1, c))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"A user just approved an unexpected command: {approved_command}

There is no active project agenda. Ask 2-3 brief questions to understand what the user is working on.

Recent commands:
{history_text}

Respond in this EXACT JSON format:
```json
{{
  "questions": [
    {{"question": "What are you working on?", "type": "text"}},
    {{"question": "Which area of the codebase?", "type": "text"}},
    {{"question": "Will you need more package installs?", "type": "choice", "options": ["yes", "no", "maybe"]}}
  ]
}}
```

Keep questions short and practical."#
    )
}

#[derive(Debug, serde::Deserialize)]
pub struct ParsedDeliberation {
    pub decision: String,
    pub confidence: f64,
    pub reason: String,
}

pub fn parse_deliberation(raw: &str) -> Option<ParsedDeliberation> {
    let mut decision = None;
    let mut confidence = None;
    let mut reason = None;

    for line in raw.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("DECISION:") {
            decision = Some(val.trim().to_uppercase());
        } else if let Some(val) = line.strip_prefix("CONFIDENCE:") {
            confidence = val.trim().parse::<f64>().ok();
        } else if let Some(val) = line.strip_prefix("REASON:") {
            reason = Some(val.trim().to_string());
        }
    }

    match (decision, confidence, reason) {
        (Some(d), Some(c), Some(r)) => Some(ParsedDeliberation {
            decision: d,
            confidence: c,
            reason: r,
        }),
        _ => None,
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ParsedQuestions {
    pub questions: Vec<ParsedQuestion>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ParsedQuestion {
    pub question: String,
    #[serde(rename = "type")]
    pub q_type: String,
    #[serde(default)]
    pub options: Vec<String>,
}

pub fn parse_questions(raw: &str) -> Option<ParsedQuestions> {
    let json_start = raw.find('{')?;
    let json_str = &raw[json_start..];
    serde_json::from_str(json_str).ok()
}

#[derive(Debug, serde::Deserialize)]
pub struct ParsedRule {
    pub command_pattern: String,
    pub args_pattern: String,
    pub path_pattern: Option<String>,
    pub notification: String,
    pub reason: Option<String>,
    pub confidence: Option<f64>,
}

pub fn parse_rules(raw: &str) -> Option<Vec<ParsedRule>> {
    let json_start = raw.find('[')?;
    let json_str = &raw[json_start..];
    serde_json::from_str(json_str).ok()
}
