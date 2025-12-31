pub fn wrap_template(title: &str, body_content: &str) -> String {
    format!(
r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }}
        pre {{
            background-color: #f6f8fa;
            padding: 16px;
            border-radius: 6px;
            overflow: auto;
        }}
        code {{
            font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, Courier, monospace;
            background-color: #f6f8fa;
            padding: 0.2em 0.4em;
            border-radius: 3px;
        }}
        pre code {{
            padding: 0; /* preの中のcodeはpaddingなし */
        }}
        blockquote {{
            margin: 0;
            padding-left: 1em;
            border-left: 4px solid #dfe2e5;
            color: #6a737d;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
        }}
        th, td {{
            border: 1px solid #dfe2e5;
            padding: 6px 13px;
        }}
        hr {{
            border: none;
            border-bottom: 1px solid #dfe2e5;
        }}
        input[type="checkbox"] {{
            margin-right: 0.5em;
        }}
        ul.task-list {{
            list-style-type: none;
            padding-left: 0;
        }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
        title, body_content
    )
}
