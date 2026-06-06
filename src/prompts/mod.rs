/// Prompt 参数结构
#[derive(Debug, Clone, Default)]
pub struct PromptParams {
    pub context: Option<String>,
    pub file: Option<String>,
    pub phase_name: Option<String>,
    pub plan_file: Option<String>,
    pub name: Option<String>,
}

impl PromptParams {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    #[allow(dead_code)]
    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    #[allow(dead_code)]
    pub fn with_phase_name(mut self, name: String) -> Self {
        self.phase_name = Some(name);
        self
    }

    #[allow(dead_code)]
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
/// 返回 Result<String>，如果模板中包含未提供的占位符则返回错误
pub fn render(template: &str, params: &PromptParams) -> Result<String, String> {
    // 验证所有占位符都有对应的值
    if template.contains("{context}") && params.context.is_none() {
        return Err("Missing required parameter: {context}".to_string());
    }
    if template.contains("{file}") && params.file.is_none() {
        return Err("Missing required parameter: {file}".to_string());
    }
    if template.contains("{Phase Name}") && params.phase_name.is_none() {
        return Err("Missing required parameter: {Phase Name}".to_string());
    }
    if template.contains("{plan_file}") && params.plan_file.is_none() {
        return Err("Missing required parameter: {plan_file}".to_string());
    }
    if template.contains("{name}") && params.name.is_none() {
        return Err("Missing required parameter: {name}".to_string());
    }

    let mut result = template.to_string();

    if let Some(ref context) = params.context {
        result = result.replace("{context}", context);
    }
    if let Some(ref file) = params.file {
        result = result.replace("{file}", file);
    }
    if let Some(ref phase_name) = params.phase_name {
        result = result.replace("{Phase Name}", phase_name);
    }
    if let Some(ref plan_file) = params.plan_file {
        result = result.replace("{plan_file}", plan_file);
    }
    if let Some(ref name) = params.name {
        result = result.replace("{name}", name);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic() {
        let template = "Hello {context}";
        let params = PromptParams::new().with_context("world".to_string());
        let result = render(template, &params).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_render_all_placeholders() {
        let template = "{context} {file} {Phase Name} {plan_file} {name}";
        let params = PromptParams::new()
            .with_context("c".to_string())
            .with_file("f".to_string())
            .with_phase_name("pn".to_string())
            .with_plan_file("pf".to_string())
            .with_name("n".to_string());
        let result = render(template, &params).unwrap();
        assert_eq!(result, "c f pn pf n");
    }

    #[test]
    fn test_render_missing_context() {
        let template = "Hello {context}!";
        let params = PromptParams::new();
        let result = render(template, &params);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing required parameter: {context}");
    }

    #[test]
    fn test_render_partial_missing() {
        let template = "File: {file}, Name: {name}";
        let params = PromptParams::new().with_file("test.rs".to_string());
        let result = render(template, &params);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing required parameter: {name}");
    }

    #[test]
    fn test_render_all_provided() {
        let template = "Context: {context}, File: {file}, Name: {name}";
        let params = PromptParams::new()
            .with_context("test context".to_string())
            .with_file("test.rs".to_string())
            .with_name("test".to_string());
        let result = render(template, &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Context: test context, File: test.rs, Name: test");
    }

    #[test]
    fn test_render_no_placeholders() {
        let template = "Hello world!";
        let params = PromptParams::new();
        let result = render(template, &params).unwrap();
        assert_eq!(result, "Hello world!");
    }
}
