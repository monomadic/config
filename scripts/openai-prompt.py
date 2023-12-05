#!python3

import os
import sys

import openai

# Retrieve API key from environment variable
api_key = os.getenv("OPENAI_API_KEY")
if not api_key:
    raise ValueError("The OPENAI_API_KEY environment variable is not set.")

openai.api_key = api_key

# Get command-line arguments as prompt, excluding the script name
if len(sys.argv) < 2:
    raise ValueError(
        "No prompt provided. Please provide a prompt as a command-line argument."
    )
prompt = " ".join(sys.argv[1:])

response = openai.ChatCompletion.create(
    model="gpt-4",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": prompt},
    ],
)

print(response.choices[0].message["content"])
