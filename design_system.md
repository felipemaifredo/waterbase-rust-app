# Design System - Waterbase HUD

This design system defines the visual guidelines, color palette, and component structure for the Waterbase HUD and Login screen. The interface is inspired by a modern, high-premium dark-mode aesthetic (Vercel/Linear style) featuring deep dark grays and vibrant neon blue accents.

## 🎨 Color Palette

| Name | Hex Value | Purpose |
| :--- | :--- | :--- |
| **Background (Deep)** | `#09090b` | Main application background |
| **Surface (Card)** | `#18181b` | Login card, collection details, document rows |
| **Border** | `#27272a` | Dividers, card borders, active states |
| **Primary Accent (Neon Blue)**| `#00d2ff` | Links, active buttons, focus indicators, neon effects |
| **Text Primary** | `#f4f4f5` | Main headings, active inputs, readable content |
| **Text Secondary** | `#a1a1aa` | Labels, placeholders, timestamps |
| **Success** | `#10b981` | Positive actions (Document created, connected) |
| **Danger** | `#ef4444` | Negative actions (Delete, disconnect, errors) |

## 🔤 Typography

- **Font Family**: `Inter`, `-apple-system`, `BlinkMacSystemFont`, `"Segoe UI"`, `Roboto`, `sans-serif`
- **Headings**: Semibold/Bold, tracking tight (`letter-spacing: -0.025em`)
- **Body**: Regular, zinc-400 for secondary, zinc-100 for primary.

## 🖥️ Layout & Components

### 1. Login Screen
- **Container**: Centered box with glassmorphism effects (`backdrop-filter`, subtle border, shadow).
- **Inputs**: Dark background (`#09090b`), subtle gray border, glows neon blue on `:focus`.
- **Button**: Solid primary accent color with smooth transition, turning white on hover.

### 2. HUD Dashboard
- **Sidebar**: List of collections, with a button to "+" create a new collection.
- **Main Area**:
  - Top header displaying the active collection.
  - A button to "+" create a new document.
  - A table or grid listing all documents with their IDs and JSON fields.
  - Action buttons (Edit, Delete) inline.
- **Fields Form / Modal**: Interactive fields builder where keys and values can be edited.
- **Alerts/Toasts**: Minimal popups in the corner showing operation success/error.

## ✨ Interactions & Micro-animations

- **Buttons/Links**: Hover states shift opacity or brightness with a transition of `0.2s ease`.
- **Hover Cards**: Border transitions from zinc-800 to zinc-700 on hover.
- **Focus Rings**: Standard outline replaced by `box-shadow: 0 0 0 2px rgba(0, 210, 255, 0.4)`.
