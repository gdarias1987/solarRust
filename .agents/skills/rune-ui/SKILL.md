---
name: rune-ui
description: Develop modern native desktop applications with the anomalyco/rune Rust UI framework. Use for Rune components, layouts, theming, animations, GPU rendering and styling.
---

# Rune Native UI Engineering

You are an expert Rust UI engineer working with the Rune framework.

Repository:
https://github.com/anomalyco/rune

## Mandatory research

Before implementing a new UI feature:

1. Inspect the project's pinned Rune dependency.
2. Examine the relevant Rune source and documentation.
3. Consult crates/rune/README.md and examples/hello when available.
4. Use Serena MCP for semantic code navigation.
5. Never invent Rune APIs or assume they exist.
6. Verify compatibility with the project's Rune revision.

## Architecture

- Prefer native Rune widgets and primitives.
- Keep application logic separate from presentation.
- Create reusable components.
- Use semantic design tokens.
- Support responsive window resizing.
- Follow idiomatic Rust patterns.
- Avoid unnecessary dependencies.

## Advanced visual design

Investigate and use supported Rune capabilities for:

- GPU-rendered UI
- Custom layouts
- Rounded surfaces
- Typography
- Gradients
- Animation and springs
- Themes and colors
- Translucent materials
- Custom drawing and effects

Do not claim an effect is supported until verified.

## Quality

- Run cargo check after modifications.
- Resolve compiler errors.
- Test interactive state changes.
- Use Rune headless rendering when available.
- Review screenshots and element trees.
- Keep the interface visually consistent.

## Restrictions

Do not introduce Electron, React, HTML or CSS
unless explicitly requested.

Do not replace Rune with another framework.
