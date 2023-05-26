# list all available models
function ai-ls-models() {
	curl -H "Authorization: Bearer $OPENAI_API_KEY" \
		 https://api.openai.com/v1/models \
		 | jq -r '.data[].id'
}

export DEFAULT_GPT_MODEL="gpt-3.5-turbo"
export PROMPT_DECOMPILER_OUTPUT="Make the following code more readable. Add comments where appropriate, and move variables into appropriate scope."

alias ai-decompiler=""
