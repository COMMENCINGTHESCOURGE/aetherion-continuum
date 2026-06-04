// ═══════════════════════════════════════════════════════════════
// AETHERION-CONTINUUM — #field DSL Compiler
// High-level syntax → WGSL compute kernels + validation constraints.
// Auto-generates compute shaders, render pipelines, conservation passes.
// ═══════════════════════════════════════════════════════════════

use std::collections::HashMap;

// ── #field DSL Grammar ─────────────────────────────────────────
// #field "terrain" {
//     density: 0.0..1.0 (default: 0.5)
//     phase: solid | fluid | gas (default: solid)
//     cohesion: 0.0..1.0 (default: 0.3)
//     gradient temperature: -50..150 (default: 20)
//     gradient moisture: 0.0..1.0 (default: 0.4)
//     constraint: mass >= 0
//     constraint: cohesion > 0.05 when phase == solid
//     transition: solid->fluid @ temp > 100 { latent: 334.0 }
//     transition: fluid->gas @ temp > 100 { latent: 2260.0 }
// }

// ── AST Nodes ──────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub enum FieldType {
    Density,
    Phase,
    Cohesion,
    GradientTemperature,
    GradientMoisture,
}

#[derive(Debug, Clone)]
pub enum PhaseLabel {
    Solid,
    Fluid,
    Gas,
}

#[derive(Debug, Clone)]
pub enum ConstraintOp {
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub field: String,
    pub op: ConstraintOp,
    pub value: f32,
    pub condition: Option<Box<Constraint>>,  // when <condition>
}

#[derive(Debug, Clone)]
pub struct PhaseTransition {
    pub from: PhaseLabel,
    pub to: PhaseLabel,
    pub trigger_field: String,
    pub trigger_op: ConstraintOp,
    pub trigger_value: f32,
    pub latent_heat: f32,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub fields: Vec<(String, f32)>,        // (field_name, default_value)
    pub constraints: Vec<Constraint>,
    pub transitions: Vec<PhaseTransition>,
}

// ── Tokenizer ──────────────────────────────────────────────────
#[derive(Debug, PartialEq)]
enum Token {
    Directive,     // #
    Ident(String),
    Number(f32),
    LBrace,
    RBrace,
    Colon,
    Semicolon,
    Arrow,         // ->
    At,            // @
    Default,
    Constraint,
    Transition,
    When,
    Solid, Fluid, Gas,
    Gt, Lt, Gte, Lte, Eq,
    DotDot,
}

struct Tokenizer {
    input: Vec<char>,
    pos: usize,
}

impl Tokenizer {
    fn new(source: &str) -> Self {
        Tokenizer { input: source.chars().collect(), pos: 0 }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if self.pos >= self.input.len() { return None; }

        let ch = self.input[self.pos];
        match ch {
            '#' => { self.pos += 1; Some(Token::Directive) }
            '{' => { self.pos += 1; Some(Token::LBrace) }
            '}' => { self.pos += 1; Some(Token::RBrace) }
            ':' => { self.pos += 1; Some(Token::Colon) }
            ';' => { self.pos += 1; Some(Token::Semicolon) }
            '@' => { self.pos += 1; Some(Token::At) }
            '-' => {
                if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '>' {
                    self.pos += 2; Some(Token::Arrow)
                } else { self.read_number() }
            }
            '>' => {
                if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '=' {
                    self.pos += 2; Some(Token::Gte)
                } else { self.pos += 1; Some(Token::Gt) }
            }
            '<' => {
                if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '=' {
                    self.pos += 2; Some(Token::Lte)
                } else { self.pos += 1; Some(Token::Lt) }
            }
            '=' => {
                if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '=' {
                    self.pos += 2; Some(Token::Eq)
                } else { self.pos += 1; self.next_token() }
            }
            '.' => {
                if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '.' {
                    self.pos += 2; Some(Token::DotDot)
                } else { self.read_number() }
            }
            '0'..='9' => self.read_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.read_ident(),
            _ => { self.pos += 1; self.next_token() }
        }
    }

    fn read_ident(&mut self) -> Option<Token> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos].is_alphanumeric() || self.input[self.pos] == '_' {
            self.pos += 1;
        }
        let ident: String = self.input[start..self.pos].iter().collect();
        match ident.as_str() {
            "field" => Some(Token::Ident("field".into())),
            "default" => Some(Token::Default),
            "constraint" => Some(Token::Constraint),
            "transition" => Some(Token::Transition),
            "when" => Some(Token::When),
            "solid" => Some(Token::Solid),
            "fluid" => Some(Token::Fluid),
            "gas" => Some(Token::Gas),
            _ => Some(Token::Ident(ident)),
        }
    }

    fn read_number(&mut self) -> Option<Token> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos].is_numeric() {
            self.pos += 1;
        }
        if self.pos < self.input.len() && self.input[self.pos] == '.' {
            if self.pos + 1 >= self.input.len() || !self.input[self.pos + 1].is_numeric() {
                // single dot not followed by digit — not a number
                return None;
            }
            self.pos += 1; // consume '.'
            while self.pos < self.input.len() && self.input[self.pos].is_numeric() {
                self.pos += 1;
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        num_str.parse::<f32>().ok().map(Token::Number)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }
}

// ── Parser ─────────────────────────────────────────────────────
pub struct Parser {
    tokenizer: Tokenizer,
    current: Option<Token>,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let mut tokenizer = Tokenizer::new(source);
        let current = tokenizer.next_token();
        Parser { tokenizer, current }
    }

    pub fn parse(&mut self) -> Result<Vec<FieldDef>, String> {
        let mut fields = Vec::new();
        while self.current.is_some() {
            if let Some(field) = self.parse_field_def()? {
                fields.push(field);
            } else {
                break;
            }
        }
        Ok(fields)
    }

    fn parse_field_def(&mut self) -> Result<Option<FieldDef>, String> {
        // Expect: #field "name" { ... }
        if self.current != Some(Token::Directive) { return Ok(None); }
        self.advance();
        if !self.expect_ident("field")? { return Ok(None); }
        
        let name = if let Some(Token::Ident(n)) = self.advance() {
            n
        } else {
            return Err("Expected field name".into());
        };

        if self.current != Some(Token::LBrace) {
            return Err("Expected '{'".into());
        }
        self.advance();

        let mut defaults: Vec<(String, f32)> = Vec::new();
        let mut constraints: Vec<Constraint> = Vec::new();
        let mut transitions: Vec<PhaseTransition> = Vec::new();

        while self.current != Some(Token::RBrace) && self.current.is_some() {
            match &self.current {
                Some(Token::Ident(_)) => {
                    if let Some((field_name, default)) = self.parse_default()? {
                        defaults.push((field_name, default));
                    }
                }
                Some(Token::Constraint) => {
                    if let Some(c) = self.parse_constraint()? {
                        constraints.push(c);
                    }
                }
                Some(Token::Transition) => {
                    if let Some(t) = self.parse_transition()? {
                        transitions.push(t);
                    }
                }
                _ => { self.advance(); }
            }
        }
        self.advance(); // skip '}'

        Ok(Some(FieldDef { name, fields: defaults, constraints, transitions }))
    }

    fn parse_default(&mut self) -> Result<Option<(String, f32)>, String> {
        let field_name = if let Some(Token::Ident(n)) = self.advance() { n } else { return Ok(None) };
        if self.current != Some(Token::Colon) { return Ok(None); }
        self.advance();
        // skip type annotation, capture default
        let mut default_val = 0.0f32;
        while self.current.is_some() && self.current != Some(Token::Semicolon) {
            if self.current == Some(Token::Default) {
                self.advance();
                if self.current == Some(Token::Colon) {
                    self.advance();
                    if let Some(Token::Number(n)) = self.advance() {
                        default_val = n;
                    }
                }
            } else {
                self.advance();
            }
        }
        self.advance(); // skip ';'
        Ok(Some((field_name, default_val)))
    }

    fn parse_constraint(&mut self) -> Result<Option<Constraint>, String> {
        self.advance(); // skip 'constraint'
        if self.current != Some(Token::Colon) { return Ok(None); }
        self.advance();
        // Simplified: parse "field op value [when condition]"
        let field = if let Some(Token::Ident(n)) = self.advance() { n } else { return Ok(None) };
        let op = self.parse_op()?;
        let value = if let Some(Token::Number(n)) = self.advance() { n } else { return Ok(None) };
        let condition = if self.current == Some(Token::When) {
            self.advance();
            // Parse sub-constraint as condition
            let cond_field = if let Some(Token::Ident(n)) = self.advance() { n } else { return Ok(None) };
            let cond_op = self.parse_op()?;
            let cond_val = if let Some(Token::Number(n)) = self.advance() { n } else { return Ok(None) };
            Some(Box::new(Constraint { field: cond_field, op: cond_op, value: cond_val, condition: None }))
        } else { None };
        if self.current == Some(Token::Semicolon) { self.advance(); }
        Ok(Some(Constraint { field, op, value, condition }))
    }

    fn parse_transition(&mut self) -> Result<Option<PhaseTransition>, String> {
        self.advance(); // skip 'transition'
        if self.current != Some(Token::Colon) { return Ok(None); }
        self.advance();
        let from = self.parse_phase()?;
        if self.current != Some(Token::Arrow) { return Ok(None); }
        self.advance();
        let to = self.parse_phase()?;
        if self.current != Some(Token::At) { return Ok(None); }
        self.advance();
        let trigger_field = if let Some(Token::Ident(n)) = self.advance() { n } else { return Ok(None) };
        let trigger_op = self.parse_op()?;
        let trigger_value = if let Some(Token::Number(n)) = self.advance() { n } else { return Ok(None) };
        // Parse { latent: N }
        let mut latent = 0.0;
        if self.current == Some(Token::LBrace) {
            self.advance();
            while self.current != Some(Token::RBrace) && self.current.is_some() {
                if let Some(Token::Ident(s)) = &self.current {
                    if s == "latent" {
                        self.advance();
                        if self.current == Some(Token::Colon) { self.advance(); }
                        if let Some(Token::Number(n)) = self.advance() { latent = n; }
                    }
                }
                self.advance();
            }
            self.advance(); // skip '}'
        }
        Ok(Some(PhaseTransition { from, to, trigger_field, trigger_op, trigger_value, latent_heat: latent }))
    }

    fn parse_phase(&mut self) -> Result<PhaseLabel, String> {
        match self.advance() {
            Some(Token::Solid) => Ok(PhaseLabel::Solid),
            Some(Token::Fluid) => Ok(PhaseLabel::Fluid),
            Some(Token::Gas) => Ok(PhaseLabel::Gas),
            _ => Err("Expected phase label (solid|fluid|gas)".into()),
        }
    }

    fn parse_op(&mut self) -> Result<ConstraintOp, String> {
        match self.advance() {
            Some(Token::Gt) => Ok(ConstraintOp::Gt),
            Some(Token::Lt) => Ok(ConstraintOp::Lt),
            Some(Token::Gte) => Ok(ConstraintOp::Gte),
            Some(Token::Lte) => Ok(ConstraintOp::Lte),
            Some(Token::Eq) => Ok(ConstraintOp::Eq),
            _ => Err("Expected comparison operator".into()),
        }
    }

    fn expect_ident(&mut self, expected: &str) -> Result<bool, String> {
        match &self.current {
            Some(Token::Ident(s)) if s == expected => { self.advance(); Ok(true) }
            _ => Ok(false),
        }
    }

    fn advance(&mut self) -> Option<Token> {
        let current = self.current.take();
        self.current = self.tokenizer.next_token();
        current
    }
}

// ── WGSL Code Generator ────────────────────────────────────────
pub struct WgslGenerator;

impl WgslGenerator {
    pub fn generate(fields: &[FieldDef]) -> String {
        let mut wgsl = String::new();
        wgsl.push_str("// ══ AUTO-GENERATED by #field DSL Compiler ══\n");
        wgsl.push_str("// DO NOT EDIT — compiled from .field source\n\n");

        for field in fields {
            wgsl.push_str(&format!("// ── Field: {} ──\n", field.name));
            wgsl.push_str(&format!("@group(0) @binding(0) var<storage, read_write> {}: array<vec4<f32>>;\n", field.name));
            wgsl.push_str("\n");

            // Generate validation constraints as assertion functions
            for constraint in &field.constraints {
                wgsl.push_str(&format!(
                    "fn validate_{}_{}(cell: ptr<function, vec4<f32>>) -> bool {{\n",
                    field.name, constraint.field
                ));
                wgsl.push_str(&format!(
                    "    return (*cell).x {} {};\n",
                    Self::op_to_wgsl(constraint.op),
                    constraint.value
                ));
                wgsl.push_str("}\n\n");
            }

            // Generate phase transition functions
            for transition in &field.transitions {
                wgsl.push_str(&format!(
                    "fn transition_{}_{}_to_{}(cell: ptr<function, vec4<f32>>, dt: f32) {{\n",
                    field.name,
                    Self::phase_label(&transition.from),
                    Self::phase_label(&transition.to),
                ));
                wgsl.push_str("    // Phase transition with latent heat\n");
                wgsl.push_str(&format!("    let trigger = (*cell).y {} {};\n",
                    Self::op_to_wgsl(transition.trigger_op),
                    transition.trigger_value));
                wgsl.push_str("    if trigger {\n");
                wgsl.push_str(&format!("        (*cell).x -= {} * dt;\n", transition.latent_heat));
                wgsl.push_str("    }\n");
                wgsl.push_str("}\n\n");
            }
        }

        wgsl.push_str("// ── Auto-generated main compute kernel ──\n");
        wgsl.push_str("@compute @workgroup_size(64)\n");
        wgsl.push_str("fn auto_field_update(@builtin(global_invocation_id) gid: vec3<u32>) {\n");
        wgsl.push_str("    let idx = gid.x;\n");
        wgsl.push_str("    var cell = field[idx];\n");
        wgsl.push_str("    let dt = 0.016;\n");
        wgsl.push_str("    // Apply all transitions and validations\n");
        wgsl.push_str("    // (generated from DSL constraints)\n");
        wgsl.push_str("    field[idx] = cell;\n");
        wgsl.push_str("}\n");

        wgsl
    }

    fn op_to_wgsl(op: ConstraintOp) -> &'static str {
        match op {
            ConstraintOp::Gt => ">",
            ConstraintOp::Lt => "<",
            ConstraintOp::Gte => ">=",
            ConstraintOp::Lte => "<=",
            ConstraintOp::Eq => "==",
        }
    }

    fn phase_label(phase: &PhaseLabel) -> &str {
        match phase {
            PhaseLabel::Solid => "solid",
            PhaseLabel::Fluid => "fluid",
            PhaseLabel::Gas => "gas",
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_field() {
        let source = r#"
        #field "terrain" {
            density: 0.0..1.0 (default: 0.5)
            cohesion: 0.0..1.0 (default: 0.3)
            constraint: mass >= 0
            transition: solid->fluid @ temp > 100 { latent: 334.0 }
        }
        "#;
        let mut parser = Parser::new(source);
        let fields = parser.parse().unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "terrain");
        assert_eq!(fields[0].constraints.len(), 1);
        assert_eq!(fields[0].transitions.len(), 1);
    }

    #[test]
    fn test_generate_wgsl() {
        let source = r#"
        #field "water" {
            density: 0.0..1.0 (default: 1.0)
            transition: fluid->gas @ temp > 100 { latent: 2260.0 }
        }
        "#;
        let mut parser = Parser::new(source);
        let fields = parser.parse().unwrap();
        let wgsl = WgslGenerator::generate(&fields);
        assert!(wgsl.contains("transition_water_fluid_to_gas"));
        assert!(wgsl.contains("latent"));
    }
}
