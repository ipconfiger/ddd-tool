/// Prompt 参数结构
#[derive(Debug, Clone, Default)]
pub struct PromptParams {
    pub context: Option<String>,
    pub file: Option<String>,
    pub anem: Option<String>,
    pub phrase_name: Option<String>,
    pub plan_file: Option<String>,
    pub name: Option<String>,
}

impl PromptParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_anem(mut self, anem: String) -> Self {
        self.anem = Some(anem);
        self
    }

    pub fn with_phrase_name(mut self, name: String) -> Self {
        self.phrase_name = Some(name);
        self
    }

    pub fn with_plan_file(mut self, plan_file: String) -> Self {
        self.plan_file = Some(plan_file);
        self
    }

    #[allow(dead_code)]
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

/// 渲染 Prompt 模板
/// 安全替换占位符，严禁修改 Prompt 原文结构
pub fn render(template: &str, params: &PromptParams) -> String {
    let mut result = template.to_string();

    if let Some(ref context) = params.context {
        result = result.replace("{context}", context);
    }
    if let Some(ref file) = params.file {
        result = result.replace("{file}", file);
    }
    if let Some(ref anem) = params.anem {
        result = result.replace("{anem}", anem);
    }
    if let Some(ref phrase_name) = params.phrase_name {
        result = result.replace("{Phrase Name}", phrase_name);
    }
    if let Some(ref plan_file) = params.plan_file {
        result = result.replace("{plan_file}", plan_file);
    }
    if let Some(ref name) = params.name {
        result = result.replace("{name}", name);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic() {
        let template = "Hello {context}";
        let params = PromptParams::new().with_context("world".to_string());
        let result = render(template, &params);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_render_all_placeholders() {
        let template = "{context} {file} {anem} {Phrase Name} {plan_file} {name}";
        let params = PromptParams::new()
            .with_context("c".to_string())
            .with_file("f".to_string())
            .with_anem("a".to_string())
            .with_phrase_name("pn".to_string())
            .with_plan_file("pf".to_string())
            .with_name("n".to_string());
        let result = render(template, &params);
        assert_eq!(result, "c f a pn pf n");
    }

    #[test]
    fn test_render_preserves_unchanged() {
        let template = "Hello {context}!";
        let params = PromptParams::new();
        let result = render(template, &params);
        assert_eq!(result, "Hello {context}!");
    }
}
