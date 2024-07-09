#!/usr/bin/env zsh

# Function to check if a command exists
command_exists() {
  command -v "$1" >/dev/null 2>&1
}

# Check for required commands
for cmd in curl dig nc; do
  if ! command_exists $cmd; then
    echo "Error: $cmd is not installed. Please install it and try again."
    exit 1
  fi
done

# Define variables
TEST_URL="http://connectivitycheck.gstatic.com/generate_204"
DNS_TEST_DOMAIN="www.google.com"
TIMEOUT=5

echo "Checking for captive portal..."

# 1. DNS resolution check
if ! dig +short $DNS_TEST_DOMAIN >/dev/null 2>&1; then
  echo "DNS resolution failed. Possible captive portal."
else
  echo "DNS resolution successful."
fi

# 2 & 3. HTTP request and response analysis
RESPONSE=$(curl -sS -m $TIMEOUT -o /dev/null -w "%{http_code} %{redirect_url}" $TEST_URL)
STATUS_CODE=$(echo $RESPONSE | cut -d' ' -f1)
REDIRECT_URL=$(echo $RESPONSE | cut -d' ' -f2-)

if [[ $STATUS_CODE != "204" ]]; then
  echo "Unexpected status code: $STATUS_CODE. Possible captive portal."
  if [[ -n $REDIRECT_URL ]]; then
    echo "Captive portal URL: $REDIRECT_URL"
  fi
else
  echo "HTTP check passed. No captive portal detected."
fi

# 4. Connectivity test
if ! nc -z -w $TIMEOUT www.google.com 80 >/dev/null 2>&1; then
  echo "TCP connection test failed. Possible captive portal."
else
  echo "TCP connection test passed."
fi

echo "Captive portal detection complete."
