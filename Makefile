default: jpv.zip

jpv.zip: extension/*
	cd extension && zip -1 -r ../jpv.zip * --exclude '*.git*'
