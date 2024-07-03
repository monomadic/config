# Dotfiles

## MacOS (Apple Silicon)

```bash
# checkout dotfiles repository
git clone https://github.com/monomadic/config $HOME/config
cd $HOME/config

# install dependencies with homebrew
bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew bundle

# link and deploy dotter configs
dotter deploy

# python deps
# pip3 install -r python.deps
```
