#!/bin/bash
END_POINT="http://0.0.0.0:3001/api/tree"
function get_tree(){
  curl -s -X GET $END_POINT | ( [ -x "$(command -v jq)" ] && jq || cat )
}

function post_node() {
  ( [ -x "$(command -v jq)" ] && (echo "$1" | jq) || echo $1 )
  curl -X POST $END_POINT -H 'Content-Type: application/json' -d "$1"
}

echo "Testing the API"
echo " "
echo "------Initial get with empty tree------"
get_tree
echo " "
echo " "
echo "------Adding a root node------"
post_node '{"label": "root"}'
echo " "
echo " "
echo "------Adding a child node------"
post_node '{"label": "child", "parent_id": 1}'
echo " "
echo " "
echo "------Get updated tree------"
get_tree
