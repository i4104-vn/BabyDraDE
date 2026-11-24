# Component Design Guidelines

## 1. Component Anatomy

UI components must follow this structure in order:

```rust
// 1. Config / Props
#[derive(Debug, Clone)]
pub struct ButtonConfig {
    pub label: String,
    pub icon: Option<String>,
    pub style: ButtonStyle,
}

// 2. Widget Struct (State & Elements)
pub struct PrimaryButton {
    root: gtk4::Button,
    label: gtk4::Label,
}

// 3. Constructor
impl PrimaryButton {
    pub fn new(cfg: &ButtonConfig) -> Self {
        let label = gtk4::Label::new(Some(&cfg.label));
        let root = gtk4::Button::new();
        root.set_child(Some(&label));
        root.add_css_class("primary-button");
        Self { root, label }
    }

    // 4. Public API
    pub fn set_label(&self, text: &str) { self.label.set_text(text); }
    pub fn widget(&self) -> &gtk4::Button { &self.root }
}
```

Rule: Connect signals (`connect_clicked`, etc.) outside the constructor, in the adapter layer.

## 2. Design Tokens

Use CSS classes instead of hardcoded values in Rust:

| Class | CSS Value | Usage |
| :--- | :--- | :--- |
| surface-primary | background: rgb(14,14,18) | Main background |
| surface-card | background: rgb(24,25,31) | Inner cards |
| border-std | border: 1px solid rgba(255,255,255,0.12) | Standard border |
| radius-capsule | border-radius: 9999px | Pills and capsules |
| radius-card | border-radius: 14px | Cards and popovers |
| text-primary | color: rgba(255,255,255,0.95) | Titles, active text |
| text-secondary | color: rgba(255,255,255,0.60) | Labels, descriptions |
| text-muted | color: rgba(255,255,255,0.40) | Disabled text |

## 3. Required States

Interactive components must support:
- Default: Normal display state.
- Hover: 150ms ease-out transition.
- Active/Pressed: Subtle scale down or border highlight.
- Disabled: `set_sensitive(false)`, opacity 0.4.

## 4. Animation

- Use `libs/animation` (spring physics) for complex transitions.
- Use CSS transitions only for simple micro-interactions (opacity, color).
- Recommended durations:
  - Micro-interactions: 120ms - 180ms
  - Popovers: 200ms - 280ms
  - Workspace switch: 300ms - 350ms
