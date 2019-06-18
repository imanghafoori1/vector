# cat Cargo.toml | grep "version" | head -n 1 | cut -f3 -d' ' | tr -d '"'
# git describe --abbrev=0 --tags

if [ -n "$CIRCLE_TAG" ]
then
  git describe --abbrev=0 --tags
elif [ -n "$CIRCLE_BRANCH" ]
then
  git describe --tags
else
  echo "error: neither TAG nor BRANCH was set"
  exit 1
fi