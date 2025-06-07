# このディレクトリが置いてある場所を指定して
path="/Users/jr/cli/image-resize-convert-uploader/"

cd $path
./target/debug/image-resize-convert-uploader

# githubにpushするならコメントアウト解除して
git add .
git commit -m "画像追加"
git push origin master