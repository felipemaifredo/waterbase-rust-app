pub fn get_css() -> String {
    r#"
    :root {
        --bg: #09090b;
        --surface: #18181b;
        --border: #27272a;
        --primary: #00d2ff;
        --text-primary: #f4f4f5;
        --text-secondary: #a1a1aa;
        --success: #10b981;
        --danger: #ef4444;
    }

    * {
        box-sizing: border-box;
    }

    body, input, textarea, select, button {
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    }

    body {
        background-color: var(--bg);
        color: var(--text-primary);
        margin: 0;
        padding: 0;
        display: flex;
        height: 100vh;
        overflow: hidden;
    }

    /* Login Page Styling */
    .login-body {
        display: flex;
        justify-content: center;
        align-items: center;
        width: 100vw;
        height: 100vh;
        background-color: var(--bg);
    }

    .login-card {
        background-color: var(--surface);
        border: 1px solid var(--border);
        border-radius: 12px;
        padding: 40px;
        width: 360px;
        box-shadow: 0 4px 24px rgba(0, 0, 0, 0.5);
    }

    .login-card h1 {
        margin: 0 0 8px 0;
        font-size: 28px;
        font-weight: 700;
        letter-spacing: -0.05em;
        text-align: center;
        background: linear-gradient(135deg, #ffffff 0%, var(--primary) 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .login-card .subtitle {
        color: var(--text-secondary);
        font-size: 14px;
        margin: 0 0 24px 0;
        text-align: center;
    }

    .input-group {
        text-align: left;
        margin-bottom: 18px;
    }

    .input-group label {
        display: block;
        font-size: 12px;
        font-weight: 600;
        color: var(--text-secondary);
        margin-bottom: 6px;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .input-group input, .input-group textarea, .input-group select {
        width: 100%;
        box-sizing: border-box;
        background-color: var(--bg);
        border: 1px solid var(--border);
        border-radius: 6px;
        padding: 10px 12px;
        color: var(--text-primary);
        font-size: 14px;
        transition: border-color 0.2s, box-shadow 0.2s;
    }

    .input-group input:focus, .input-group textarea:focus, .input-group select:focus {
        outline: none;
        border-color: var(--primary);
        box-shadow: 0 0 0 2px rgba(0, 210, 255, 0.2);
    }

    .error-msg {
        color: var(--danger);
        font-size: 13px;
        margin: 10px 0;
        text-align: left;
    }

    .btn-primary {
        width: 100%;
        background-color: var(--primary);
        color: #09090b;
        border: none;
        border-radius: 6px;
        padding: 12px;
        font-size: 14px;
        font-weight: 600;
        cursor: pointer;
        transition: background-color 0.2s, opacity 0.2s;
    }

    .btn-primary:hover {
        opacity: 0.9;
    }

    /* Dashboard Panel Styling */
    .app-container {
        display: flex;
        width: 100%;
        height: 100%;
    }

    .sidebar {
        width: 280px;
        background-color: var(--surface);
        border-right: 1px solid var(--border);
        display: flex;
        flex-direction: column;
        padding: 24px;
        box-sizing: border-box;
    }

    .sidebar-header {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 24px;
    }

    .sidebar-header h1 {
        font-size: 20px;
        margin: 0;
        font-weight: 700;
        letter-spacing: -0.05em;
        background: linear-gradient(135deg, #ffffff 0%, var(--primary) 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .sidebar h2 {
        font-size: 11px;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        color: var(--text-secondary);
        margin: 0 0 12px 0;
    }

    .sidebar-menu {
        flex-grow: 1;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .sidebar-item {
        padding: 10px 12px;
        border-radius: 6px;
        color: var(--text-secondary);
        text-decoration: none;
        font-size: 14px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        transition: background-color 0.2s, color 0.2s;
        cursor: pointer;
    }

    .sidebar-item:hover, .sidebar-item.active {
        background-color: var(--border);
        color: var(--text-primary);
    }

    .sidebar-item.active {
        border-left: 3px solid var(--primary);
        padding-left: 9px;
        background-color: rgba(0, 210, 255, 0.05);
    }

    .new-col-form {
        display: flex;
        gap: 8px;
        margin-top: 15px;
        border-top: 1px solid var(--border);
        padding-top: 15px;
    }

    .new-col-form input {
        flex-grow: 1;
        background-color: var(--bg);
        border: 1px solid var(--border);
        border-radius: 6px;
        padding: 8px 12px;
        color: var(--text-primary);
        font-size: 13px;
        width: 100%;
        box-sizing: border-box;
    }

    .new-col-form input:focus {
        outline: none;
        border-color: var(--primary);
    }

    .new-col-form button {
        background-color: var(--primary);
        color: #000;
        border: none;
        border-radius: 6px;
        padding: 8px 12px;
        font-weight: 600;
        cursor: pointer;
    }

    .main-area {
        flex-grow: 1;
        display: flex;
        flex-direction: column;
        height: 100%;
        background-color: var(--bg);
    }

    .header {
        height: 60px;
        border-bottom: 1px solid var(--border);
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 30px;
        background-color: var(--surface);
    }

    .header h1 {
        font-size: 16px;
        margin: 0;
        font-weight: 600;
        color: var(--text-primary);
    }

    .header-actions {
        display: flex;
        align-items: center;
        gap: 15px;
    }

    .logout-btn {
        background: transparent;
        border: 1px solid var(--border);
        color: var(--danger);
        padding: 6px 12px;
        border-radius: 6px;
        font-size: 13px;
        cursor: pointer;
        text-decoration: none;
        transition: background-color 0.2s;
    }

    .logout-btn:hover {
        background-color: rgba(239, 68, 68, 0.1);
    }

    .content-body {
        flex-grow: 1;
        padding: 30px;
        overflow-y: auto;
        box-sizing: border-box;
    }

    .welcome-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: var(--text-secondary);
        text-align: center;
    }

    .welcome-container h2 {
        color: var(--text-primary);
        margin-bottom: 8px;
    }

    .doc-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
        gap: 20px;
        align-items: start;
    }

    .doc-card {
        background-color: var(--surface);
        border: 1px solid var(--border);
        border-radius: 8px;
        padding: 20px;
        box-shadow: 0 4px 12px rgba(0,0,0,0.2);
    }

    .doc-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 12px;
        border-bottom: 1px solid var(--border);
        padding-bottom: 10px;
    }

    .doc-id {
        font-family: monospace;
        font-size: 14px;
        color: var(--primary);
        font-weight: bold;
    }

    .textarea-json {
        width: 100%;
        height: 140px;
        background-color: var(--bg);
        border: 1px solid var(--border);
        border-radius: 6px;
        padding: 10px;
        color: var(--text-primary);
        font-family: monospace;
        font-size: 13px;
        resize: vertical;
        box-sizing: border-box;
        margin-bottom: 12px;
    }

    .textarea-json:focus {
        outline: none;
        border-color: var(--primary);
    }

    .card-actions {
        display: flex;
        justify-content: space-between;
        gap: 10px;
    }

    .btn-save {
        background-color: var(--success);
        color: #000;
        border: none;
        border-radius: 6px;
        padding: 8px 16px;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
    }

    .btn-danger {
        background-color: transparent;
        border: 1px solid var(--danger);
        color: var(--danger);
        border-radius: 6px;
        padding: 8px 16px;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        transition: background-color 0.2s;
    }

    .btn-danger:hover {
        background-color: rgba(239, 68, 68, 0.1);
    }

    .create-doc-card {
        background-color: var(--surface);
        border: 1px dashed var(--border);
        border-radius: 8px;
        padding: 20px;
        margin-bottom: 30px;
    }

    .create-doc-card h3 {
        margin: 0 0 15px 0;
        font-size: 15px;
        font-weight: 600;
    }

    .inline-fields {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .inline-fields input {
        background-color: var(--bg);
        border: 1px solid var(--border);
        border-radius: 6px;
        padding: 8px 12px;
        color: var(--text-primary);
        font-size: 13px;
    }

    .inline-fields input:focus {
        outline: none;
        border-color: var(--primary);
    }

    /* Swagger API Docs CSS */
    .badge {
        display: inline-block;
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 11px;
        font-weight: bold;
        font-family: monospace;
        text-transform: uppercase;
        margin-right: 12px;
    }
    .badge-get {
        background-color: rgba(0, 210, 255, 0.15);
        color: var(--primary);
        border: 1px solid var(--primary);
    }
    .badge-post {
        background-color: rgba(16, 185, 129, 0.15);
        color: var(--success);
        border: 1px solid var(--success);
    }
    .badge-put {
        background-color: rgba(245, 158, 11, 0.15);
        color: #f59e0b;
        border: 1px solid #f59e0b;
    }
    .badge-delete {
        background-color: rgba(239, 68, 68, 0.15);
        color: var(--danger);
        border: 1px solid var(--danger);
    }
    .api-endpoint {
        background-color: var(--surface);
        border: 1px solid var(--border);
        border-radius: 8px;
        padding: 24px;
        margin-bottom: 25px;
    }
    .api-header {
        display: flex;
        align-items: center;
        margin-bottom: 12px;
    }
    .api-path {
        font-family: monospace;
        font-size: 15px;
        font-weight: 600;
        color: var(--text-primary);
    }
    .api-description {
        color: var(--text-secondary);
        font-size: 14px;
        margin-bottom: 15px;
    }
    .api-details {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        gap: 20px;
        margin-top: 15px;
    }
    .details-box {
        background-color: var(--bg);
        border: 1px solid var(--border);
        border-radius: 6px;
        padding: 15px;
    }
    .details-box h4 {
        margin: 0 0 10px 0;
        font-size: 11px;
        text-transform: uppercase;
        color: var(--text-secondary);
        letter-spacing: 0.05em;
    }
    .code-block {
        font-family: monospace;
        font-size: 12px;
        color: var(--text-primary);
        background-color: rgba(0,0,0,0.3);
        padding: 10px;
        border-radius: 4px;
        overflow-x: auto;
        white-space: pre-wrap;
        margin: 0;
        border: 1px solid rgba(255,255,255,0.03);
    }
    "#.to_string()
}
