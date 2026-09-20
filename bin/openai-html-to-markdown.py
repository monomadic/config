#!python3

import json

import requests


def convert_html_to_markdown(html):
    url = "https://api.openai.com/v1/engines/davinci/completions"
    params = {
        "prompt": f"Convert HTML to Markdown: {html}",
        "temperature": 0.7,
        "max_tokens": 100,
    }
    response = requests.post(url, params=params)
    response.raise_for_status()
    data = json.loads(response.content)
    return data["choices"][0]["text"]


if __name__ == "__main__":
    html = """
<html>
<head>
<title>This is a title</title>
</head>
<body>
<h1>This is a heading</h1>
<p>This is a paragraph.</p>
</body>
</html>
"""
    markdown = convert_html_to_markdown(html)
    print(markdown)
