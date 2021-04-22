#!/usr/bin/env bash

set -eo pipefail

##
# Update payments-api single_immediate_payments allow list
#
# Requires that a github api token with suitable perms is
# present via envvar $GITHUB_TOKEN
#
# takes a `clientId` which will be appended to the allowlist
# if it's not already present
#
# updates the allow list and creates a pull request
#
# example:
# > ./this_script sandbox-fooclient-12456
#
# Notes:
# Mac users, your version of `diffutils` may be slightly
# outdated. Update to >= 3.7
#

# print api responses and other non-functional info to stdout
DEBUG=${DEBUG:-false}
dbg(){
    if [[ "$DEBUG" == "true"  ]]; then
        declare header=$1
        declare message=$2

        printf '\e[33m%s\e[0m\n\e[32m%s\e[0m\n\n' \
        "$header" \
        "$(
            printf %s $message | jq -cC '.' 2>/dev/null ||
            printf %s $message
        )" > /dev/stderr
    fi
}

if [ -z "$1" ]; then
    echo "clientId expected as first argument"
    exit 1
fi

client_id=$1
dbg "client_id" $client_id

if [ -z "${GITHUB_TOKEN}" ]; then
    echo "No GITHUB_TOKEN set"
    exit 1
fi

if [[ "$client_id" == "sandbox-"* ]]; then
    overlay="sandbox_v2"
else
    overlay="production_v2"
fi
dbg "overlay" $overlay

file="deploy/deployment/overlays/${overlay}/configmap_patch.yaml"
repo="truelayer/payments-api"
new_branch_name="allow_list_addition/${client_id}"

API_BASE="https://api.github.com/repos/${repo}"
AUTH_HEADER="Authorization: token ${GITHUB_TOKEN}"

##
# Write some http reponse to stdout or stderr depending on http_code.
# The http_code is expected to be on the last line of the input
#
# example:
#   echo "some response text\n200" | uncurl
#
uncurl(){
    declare caller=${FUNCNAME[1]}
    declare content=${1:-$(</dev/stdin)}

    declare -i http_code=$(printf '%b\n' $content | tail -n 1)

    declare -i lines=$(printf '%b\n' $content | wc -l)-1
    declare parsed=$(printf '%b\n' $content | head -n $lines)
    if [ $http_code -gt 202 ]; then
        printf '\e[91m%s\e[0m\n' "$parsed" > /dev/stderr
        return 1
    else
        dbg "$caller" "$parsed"
        printf %s $parsed
    fi
}

##
# Returns a github 'repository' object for the target repo
#
# https://docs.github.com/en/rest/reference/repos#get-a-repository
#
get_repo() {
    curl -w "\n%{http_code}" -s -H "${AUTH_HEADER}" "${API_BASE}" | uncurl
}

##
# Returns a github 'content' object for a given path/file
#
# https://docs.github.com/en/rest/reference/repos#get-repository-content
#
get_current_content() {
    declare file=$1
    declare branch=$2

    curl -w "\n%{http_code}" -s -H "${AUTH_HEADER}" "${API_BASE}/contents/${file}?ref=${branch}" | uncurl
}


##
# Get the latest commit from the passed branch name
#
# https://docs.github.com/en/rest/reference/git#references
#
last_commit() {
    declare branch=$1

    curl -w "\n%{http_code}" -s -H "${AUTH_HEADER}" "${API_BASE}/git/refs/heads/${branch}" | uncurl
}

##
# Create a new branch (ref) from some commit
#
# https://docs.github.com/en/rest/reference/git#create-a-reference
#
create_branch() {
    declare ref=$1 # new branch identifier
	declare sha=$2 # commit sha to branch from

    declare payload='{
        "ref": "refs/heads/'"${ref}"'",
        "sha": '"\"${sha}\""'
    }'

    curl -w "\n%{http_code}" -s -H "${AUTH_HEADER}" "${API_BASE}/git/refs" -d "${payload}" | uncurl
}

##
# Return a github 'commit' object
#
# https://docs.github.com/en/rest/reference/repos#get-a-commit
#
get_commit() {
	sha=$1 # sha of commit to find tree

    curl -w "\n%{http_code}" -s -H "${AUTH_HEADER}" "${API_BASE}/git/commits/${sha}" | uncurl
}

##
# Create and return a github 'blob' object
#
# https://docs.github.com/en/rest/reference/git#create-a-blob
#
create_content_blob() {
    declare content_base64=$(echo "$1" | base64 |  sed -z 's/\n//g')

    curl -w "\n%{http_code}" -s -H "${AUTH_HEADER}" "${API_BASE}/git/blobs" \
		-d "{\"content\":\"${content_base64}\", \"encoding\":\"base64\"}" | uncurl
}

##
# Create and return a github 'tree' object
#
# https://docs.github.com/en/rest/reference/git#create-a-tree
#
create_new_tree() {
	declare base_tree_sha=$1
	declare content_sha=$2
    declare path=$3

    declare payload='{
        "base_tree":'"\"${base_tree_sha}\""',
        "tree": [{"path":'"\"${path}\""', "mode": "100644", "type": "blob", "sha":'"\"${content_sha}\""'}]
    }'

    curl -w "\n%{http_code}" -s -H "${AUTH_HEADER}" "${API_BASE}/git/trees" -d "${payload}" | uncurl
}

##
# Create and return a github 'commit' object
#
# https://docs.github.com/en/rest/reference/git#create-a-commit
#
create_commit() {
	declare parent_sha=$1 # branch_sha
	declare tree_sha=$2

    declare payload='{
        "message":"this is a commit", "parents":['"\"${parent_sha}\""'], "tree":'"\"${tree_sha}\""'
    }'

    curl -w "\n%{http_code}" -s -H "${AUTH_HEADER}" "${API_BASE}/git/commits" -d "${payload}" | uncurl
}

##
# Push a commit to the head of some ref
#
# https://docs.github.com/en/rest/reference/git#update-a-reference
#
update_ref() {
    declare branch_name=$1
    declare commit_sha=$2

    curl -w "\n%{http_code}" -s -X PATCH -H "${AUTH_HEADER}" "${API_BASE}/git/refs/heads/${branch_name}" \
        -d "{\"sha\":\"${commit_sha}\"}" | uncurl
}

##
# Create a pull request
#
# _branch: the branch (ref) where your work is
# _base: the branch into which you would like to merge
# _title: PR title as it will appear in github
# _body: the commit msg as it will appear in github
#
# https://docs.github.com/en/rest/reference/pulls#create-a-pull-request
#
create_pr() {
    declare branch=$1
    declare base=$2
    declare title=$3
    declare body=$4

    declare payload='{
        "head":'"\"${branch}\""',
        "base":'"\"${base}\""',
        "body":'"\"${body}\""',
        "title":'"\"${title}\""'
    }'

    curl -w "\n%{http_code}" -s -H "${AUTH_HEADER}" "${API_BASE}/pulls" -d "${payload}" | uncurl
}

##
# Does the existing file containt the content we expect
#
check_content() {
    _content=$1

    if [[ "$_content" != *"single_immediate_payments_v2_client_allow_list"* ]]; then
    	echo "Oops. Doesn't look lik we have the file we're looking for"
    	return 1
    fi
}

###
# Given the configuration file and a client_id, if the client_id is not already
# present, add it and return the changed file content
#
modify_content() {
    declare content=$1
    declare client_id=$2
    declare key="single_immediate_payments_v2_client_allow_list"

    # get the current list of allowed clientId's (and remove the wrapping '"' quotes)
    declare current_value=$(echo "$content" | grep $key | cut -d " " -f 4 | sed "s/\"//g")

    # bail if our incoming client_id is already represented
    if grep -E "$client_id(,|$)" <(echo $current_value); then
        return 1
    else
        # insert our incoming client_id into the existing list of client_id's
        declare new_value="$current_value,$client_id"
        declare new_content=$(echo "$content" | sed -r "s/(^.*$key: ).*$/\1\"${new_value}\"/")

        printf "%s" "$new_content"
    fi
}

## overrides
file="README.md"
repo="neilwashere/rust-project-root"
API_BASE="https://api.github.com/repos/${repo}"
new_branch_name="testit/${client_id}"
check_content() {
    return 0
}
modify_content() {
    cat ./add_client
    # echo "added ${client_id}"
}

# Check we haven't already raised a pr for this client_id
if 2>/dev/null last_commit "$new_branch_name" 1>/dev/null; then
    echo "branch already exists! byebye"
    exit 0
fi

# Determine the default branch
default_branch=$(get_repo | jq -r .default_branch)

# check if we have anything to update first
if ! content=$(get_current_content $file $default_branch | jq -r .content | base64 -d); then
    echo "Could not find current content"
    exit 1
fi

# bail if we don't have the expected content
if ! check_content "$content"; then
    echo "Unexpected content!\n${content}"
    exit 1
fi

# bail if we don't need any modifications
if ! new_content=$(modify_content "$content" $client_id); then
    echo "nothing to change :) byebye"
    exit 0
fi

echo "Showing diff..."
echo
diff --color <(printf '%b\n' "$content") <(printf '%b\n' "$new_content") || true

echo
echo "Would you like to raise a PR for this change? [y/N]"
read answer

if [[ "$answer" != "y" ]]; then
    echo "no touchy! byebye"
    exit 0
fi

# create new 'content blob' with our modifications.
# this is safer than supplying content directly when
# creating a new 'tree'
content_sha=$(create_content_blob "$new_content" | jq -r .sha)

# Get the head commit from the default branch
last_commit_sha=$(last_commit $default_branch | jq -r .object.sha)

# Create a new branch off of the default branch
branch_sha=$(create_branch $new_branch_name $last_commit_sha | jq -r .object.sha)

# Get the 'commit' object from the newly created branch
branch_tree_sha=$(get_commit $branch_sha | jq -r .tree.sha)

# Create a 'tree' object with the new content on our branch
new_tree_sha=$(create_new_tree $branch_tree_sha $content_sha $file | jq -r .sha)

# Now 'commit' that addition to our branch
new_commit_sha=$(create_commit $branch_sha $new_tree_sha | jq -r .sha)

# move our commit to the 'HEAD' of our branch
update_ref $new_branch_name $new_commit_sha > /dev/null

echo "Raising PR..."

pr_url=$(create_pr $new_branch_name $default_branch "Adding ${client_id}" "Added ${client_id} to allow list" | jq -r .html_url)
echo "pull request: $pr_url"
