export GPT_MODEL="gpt-3.5-turbo" # 4096 tokens, cheapest, fastest, most versatile model

# list all available models
function gpt-ls-models {
	curl --silent -H "Authorization: Bearer $OPENAI_API_KEY" \
		 https://api.openai.com/v1/models \
		 | jq -r '.data[].id'
}

alias gpt-refactor="python3 $HOME/config/openai/refactor.py"
alias gpt-refactor16="GPT_MODEL=gpt-3.5-turbo-16k python3 $HOME/config/openai/refactor.py"

alias gpt-html2md="python3 $HOME/config/openai/html2md.py"
alias gpt-html2md16="GPT_MODEL=gpt-3.5-turbo-16k python3 $HOME/config/openai/html2md.py"

function gpt-input {
	local prompt=$(jq --null-input --arg input "$1" '$input')
	#local prompt=$1

	if [[ -z $prompt ]]; then
		echo "Error: Invalid prompt input." >&2
		return 1
	fi

	local endpoint="https://api.openai.com/v1/chat/completions"
	local json_data='{
		"model": "'"$GPT_MODEL"'",
		"messages": [
			{
				"role": "system",
				"content": "You are a helpful assistant."
			},
			{
				"role": "user",
				"content": '$(jq -Rs @uri <<<"$prompt")'
			}
		]
	}'

	echo $json_data|jq --slurp --raw-input .

	curl -s -H "Authorization: Bearer $OPENAI_API_KEY" \
		-H "Content-Type: application/json" \
		-d "$json_data" \
		"$endpoint"
}

function gpt-prompt {
	local prompt=$(echo "$1"| jq --slurp -R '.')
	local endpoint="https://api.openai.com/v1/chat/completions"

	curl -s -H "Authorization: Bearer $OPENAI_API_KEY" \
      -H "Content-Type: application/json" \
      -d "{\"model\": \"$GPT_MODEL\", \"messages\":[{\"role\":\"system\",\"content\":\"You are a helpful assistant.\"},{\"role\":\"user\",\"content\":\"$prompt\"}]}" \
      "https://api.openai.com/v1/chat/completions"
}

function gpt-ask {
  local prompt="$1"
	local response=$(gpt-prompt $prompt)
	echo $prompt
	echo $response

  local id=$(echo $response | jq -r '.id')
  local model=$(echo $response | jq -r '.model')

  local prompt_tokens=$(echo $response | jq -r '.usage.prompt_tokens')
  local completion_tokens=$(echo $response | jq -r '.usage.completion_tokens')
  local total_tokens=$(echo $response | jq -r '.usage.total_tokens')

  local output=$(echo $response | jq -r '.choices[0].message.content')

	echo "id:\t$id"
	echo "model:\t$model"
	echo "tokens:\tprompt=$prompt_tokens, completion=$completion_tokens, total=$total_tokens"
	echo
  echo $output
}

function gpt-commit {
	local prompt="Create a git commit message based on this git diff:"
	local diff=$(git diff --staged)
	local escaped_diff=$(echo $diff| jq --slurp --raw-input .)
	gpt-prompt $(echo "$prompt\n\n$escaped_diff")
}

export DEFAULT_GPT_MODEL="gpt-3.5-turbo"
export PROMPT_DECOMPILER_OUTPUT="Make the following code more readable. Add comments where appropriate, and move variables into appropriate scope."

alias ai-decompiler=""
