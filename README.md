# gh-token-gen

gh-token-gen is yet another GitHub Action to generate token.
This is a demonstration of Node.js action; which is written in Rust, compiled into WASM.
With a trivial feature, but it's also practical.

## Usage

```yaml
jobs:
  example:
    runs-on: ubuntu-latest
    steps:
      - id: gh-token-gen
        uses: oakcask/gh-token-gen@v2
        with:
          app-id: ${{ secrets.CLIENT_ID }}
          private-key: ${{ secrets.PRIVATE_KEY }}
```

Please check out [action.yaml](./action.yaml) for further explanation of parameters.
To utilize this GitHub Action,
it is required to [setup a GitHub App][setup] and [generate a private key][generate] for the app.

[setup]: https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/about-creating-github-apps
[generate]: https://docs.github.com/en/enterprise-cloud@latest/apps/creating-github-apps/authenticating-with-a-github-app/managing-private-keys-for-github-apps

## Technological Background

### Node.js action, not Docker container action

Running in a Docker container action is one another option to run binary executable.
Actually, [gh-token-gen was once a Docker container action in v1][v1],
but quited it. Why?

Because it just longen the duration of workflows as runner git-clone the repository,
then pulls and builds image before executing the main entry point of the executable.

Meaning it invokes cargo-build every time running the action. 
There may be still a chance the image to be cached.
But as long as you runs the action in a GitHub-hosted runner it won't come.
At least, we've never seen it.

A Node.js action, however, it runs "index.js" just after git-clone the repositry.
This is the fastest option if we can reduce the number / size of files fetched by cloning.

[v1]: https://github.com/oakcask/gh-token-gen/tree/v1.0.2
