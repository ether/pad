/**
 * This file contains all code that synchronizes the pad with the
 * image in the editor. All images are synced all the time, not just
 * images that are currently displayed.
 *
 */

/**
 * There arwe three client side representations of an image. It is
 * represented as text with attributes in the pad text. There is also
 * an abstract image representation, a dict where every value is a
 * text-serialized json object for each image. Lastly, there can also
 * be a ready, rendered image of a single image that is displayed on screen.
 */

function sketchSpaceInit() {
  this.hooks = ['aceInitInnerdocbodyHead', 'aceAttribsToClasses', 'aceCreateDomLine'];
  this.padDocument = undefined;
}

/**
 *
 * This hook inserts a bunch of Javascripts and CSS links into the editor iframe. They are used for styling of and making the image icons displayed in the main pad text clickable.
 */
sketchSpaceInit.prototype.aceInitInnerdocbodyHead = function(args) {
  args.iframeHTML.push('\'<link rel="stylesheet" type="text/css" href="/static/css/plugins/sketchSpace/ace.css"/>\'');
  args.iframeHTML.push('\'\\x3cscript type="text/javascript" src="/static/js/jquery-1.3.2.js">\\x3c/script>\'');
  args.iframeHTML.push('\'\\x3cscript type="text/javascript" src="/static/js/plugins/sketchSpace/ace_inner.js">\\x3c/script>\'');
};

sketchSpaceInit.prototype.aceAttribsToClasses = function(args) {
//  console.log(args.key + ":" + args.value);
  if (args.key == 'sketchSpaceIsImage' && args.value != "")
    return ["sketchSpaceIsImage", "sketchSpaceImageId:" + args.value];
  else if (args.key.indexOf('sketchSpaceImageObject') == 0)
    return [args.key + ":" + args.value];
  else if (args.key.indexOf('sketchSpaceImageZ;')==0)
    return [args.key + ":" + args.value];
  else if (args.key == 'sketchSpaceImageZSequence')
    return [args.key + ":" + args.value];

};

/**
 * This function is called when server side image updates are recieved.
 * The function deletes the old abstract image representation and recreates it from scratch based
 * on the pad textual representation of the image.
 * If the updated image is currently displayed, call the updateImageFromPad event handler to update the screen as well.
 */
sketchSpaceInit.prototype.aceCreateDomLine = function(args) {
  if (args.cls.indexOf('sketchSpaceIsImage') >= 0) {
    var clss = [];
    var imageObjects = {};
    var imageId = undefined;
    var argClss = args.cls.split(" ");
    var zSequence = 0;
    var orderTags=[];
    var order=[];
    // sketchSpaceImageZ_ID_OBJECTID: pos
    for (var i = 0; i < argClss.length; i++) {
      var cls = argClss[i];
      if (cls.indexOf(":") != -1) {
	var key = cls.substr(0, cls.indexOf(":"));
	var val = cls.substr(cls.indexOf(":")+1);

	if (key == "sketchSpaceImageId") {
	  clss.push("sketchSpaceImageId_" + val);
	  imageId = val;
	} else if (key == "sketchSpaceImageObject") {
	  var objId = val.substr(0, val.indexOf(":"));
	  var properties = val.substr(val.indexOf(":")+1);
	  imageObjects[objId] = unescape(properties);
	} else if (key.indexOf('sketchSpaceImageZ;')==0){

	  var idx = val.split(';');
	  var oId = key.split(';');
	  oId.shift();
	  oId.shift();
/*
	  if(oId.length != idx.length){
	    console.log("DAMMIT");
	    console.log(key);
	    console.log(val);
	  }
*/
	  orderTags.push([ key.split(';'), oId, idx ]);
//	  console.log("Wee, found a Z attribute");

	} else if (key == 'sketchSpaceImageZSequence'){
	  val = parseInt(val, 10);
	  zSequence = val;
	} else {
	  clss.push(cls);
	}

      } else {
	clss.push(cls);
      }
    }

    orderTags.sort(
      function(a,b){
	for(var i=0; i<Math.min(a[0].length, b[0].length); i++) {
	  if(a[0][i] != b[0][i]){

	    if(i==1)
	      return parseInt(a[0][i],10)>parseInt(b[0][i],10);
	    else
	      return a[0][i]>b[0][i];
	  }
	}
	return a[0][i].length > b[0][i].length;
      }
    );
/*    $.each(orderTags, function(key,val){
	     console.log(val[0][1]);
	   }
	  );
*/
    $.each(
      orderTags, function(){
	var id = this[0];
	var oId = this[1];
	var idx = this[2];
	for(var i=0; i<oId.length; i++)
	{
	  var oldIdx = order.indexOf(oId[i]);
	  if(oldIdx != -1){
	    order.splice(oldIdx, 1);
	  }
	  order.splice(idx[i], 0, oId[i]);
	}
      }
    );

//    console.log("New world order:");
//    console.log(order);

    this.editorArea.images[imageId] = {objects:imageObjects, order:order, zSequence: zSequence};
    if (this.editorArea.currentImage == imageId) {
      this.updateImageFromPad();
    }

    return [{cls: clss.join(" "), extraOpenTags: '<a class="sketchSpaceImageLink">', extraCloseTags: '</a>'}];
  }
};

/**
 *
 */
sketchSpaceInit.prototype.updateImageFromPad = function() {
  if (this.editorArea.currentImage !== undefined) {
    var currentImage = this.editorArea.images[this.editorArea.currentImage].objects;
    var order = this.editorArea.images[this.editorArea.currentImage].order;

    /* Some debug info printing:
    console.log("Image:");
    sketchSpace.editorArea.forEachObjectShape(function (shape) { console.log(shape.objId); })
    console.log("Pad:");
    for (name in currentImage)
      console.log(name);
    */

    var visited = {};
    var toDelete = {};

    // Mark all changed/deleted shapes for delation
    this.editorArea.forEachObjectShape(function (shape) {
      if (currentImage[shape.objId] === undefined) {
        toDelete[shape.objId] = shape;
      } else {
        if (shape.strRepr == currentImage[shape.objId]) {
          visited[shape.objId] = shape;
        } else {
          toDelete[shape.objId] = shape;
        }
      }
    });

    // Perform deletion. This must be done after iteration or we'll
    // miss objects unpredictably because forEachObjectShape can't
    // handle graph changes while iterating.

    for (objId in toDelete) {
      toDelete[objId].removeShape();
    }

    // Given an object ID, this function will locate the corresponding
    // text data in the abstract image description and the create a
    // true editor image shape based on that description.

    // If the visited dict already contains the specified object id,
    // it will be assumed to be unchanged and will not be touched.

    function materialize (objId) {
      if (visited[objId] === undefined) {
        var objStr = currentImage[objId];
	// FIXME: Handle that objStr is undefined here... can happen
	// if stuff changed between the loop above and this function.
        var obj = dojo.fromJson(objStr);

	var parent = sketchSpace.editorArea.surface_transform;
	if (obj.parent) parent = materialize(obj.parent);

        var shape = sketchSpace.editorArea.deserializeShape(parent, obj.shape);
	sketchSpace.editorArea.registerObjectShape(shape);

        shape.objId = objId;
        shape.strRepr = objStr;
	visited[objId] = shape;
      }
      return visited[objId];
    }

    for (var objId in currentImage)
      materialize(objId);

    $.each(
      order, function(key, val){
//	console.log(val);
	if(visited[val])
  	  visited[val].moveToFront();
      }
      );

    this.editorArea.imageUpdatedByOthers();
  }
};

sketchSpaceInit.prototype.updatePadFromImage = function() {
  if (this.editorArea.currentImage !== undefined) {
    var currentImageId = this.editorArea.currentImage;
    var currentImage = this.editorArea.images[currentImageId].objects;
    var oldOrder=this.editorArea.images[currentImageId].order;
    var zSequence=this.editorArea.images[currentImageId].zSequence;

    var visited = {};
    var update = [];

    var idx = 0;

    var newOrder=[];

    var changedOrder = [];

    var oldIds = {
    };
    $.each(oldOrder, function(key,val){
	     oldIds[val]=true;
	   }
	  );

    this.editorArea.forEachObjectShape(function (shape) {
      newOrder.push(shape.objId);
      if(shape.zOrderMoved){
	shape.zOrderMoved = undefined;
	changedOrder.push(shape.objId);
      } else if(!(shape.objId in oldIds)){
	changedOrder.push(shape.objId);
      }
      if (currentImage[shape.objId] === undefined || currentImage[shape.objId] != shape.strRepr) {
        update.push(["sketchSpaceImageObject:" + shape.objId, escape(shape.strRepr)]);
      }
      visited[shape.objId] = shape;
    });
    if(changedOrder.length){
/*      console.log('Old order');
      console.log(oldOrder);
      console.log('New order');
      console.log(newOrder);
*/
      var objIdArr = newOrder;
      var idxArr = [];
      $.each(newOrder, function(key){idxArr.push(key);});

      var objIdStr = objIdArr.join(";");
      var idxStr = idxArr.join(";");

      update.push(["sketchSpaceImageZ;"+(++zSequence)+";" + objIdStr, idxStr]);
//    console.log(["sketchSpaceImageZ;"+nextSeq()+";" + objIdStr, idxStr]);

      update.push(["sketchSpaceImageZSequence", "" + zSequence]);
    }

    for (var objId in currentImage)
      if (visited[objId] === undefined)
        update.push(["sketchSpaceImageObject:" + objId, ""]);

//    console.log(update);
    this.updatePad(currentImageId, update);
  }
};

sketchSpaceInit.prototype.updatePad = function (imageId, update) {
  var sketchSpace = this;
  padeditor.ace.callWithAce(function (ace) {
    sketchSpace.ace_updatePad(ace, imageId, update);
  }, "updatePadFromImage", true);
};

sketchSpaceInit.prototype.getImageLinkFromId = function (imageId) {
  return $(this.padDocument).find(".sketchSpaceImageId_" + imageId)[0];
};

sketchSpaceInit.prototype.selectImage = function(imageLink) {
  var imageId;
  $.each($(imageLink).attr('class').split(' '), function (idx, cls) {
    var parts = cls.split("_");
    if (parts[0] == "sketchSpaceImageId")
      imageId = parts[1];
  });

  this.padDocument = imageLink.ownerDocument;
  this.editorArea.selectImage(imageId);
  this.updateImageFromPad();
};

sketchSpaceInit.prototype.insertImage = function() {
  var sketchSpace = this;

  return padeditor.ace.callWithAce(function (ace) {
    return sketchSpace.ace_insertImage(ace);
  }, "sketchSpace", true);
};

sketchSpaceInit.prototype.ace_getImageRange = function (ace, imageId) {
  var imageLink = this.getImageLinkFromId(imageId);
  return [ace.ace_getLineAndCharForPoint({node: imageLink, index:0, maxIndex:1}),
	  ace.ace_getLineAndCharForPoint({node: imageLink, index:1, maxIndex:1})];
};

sketchSpaceInit.prototype.ace_updatePad = function (ace, imageId, update) {
  var imageRange = this.ace_getImageRange(ace, imageId);
  ace.ace_performDocumentApplyAttributesToRange(imageRange[0], imageRange[1], update);
};

sketchSpaceInit.prototype.ace_insertImage = function(ace) {
  var imageId = dojox.uuid.generateRandomUuid();
  rep = ace.ace_getRep();

  ace.ace_replaceRange(rep.selStart, rep.selEnd, "I");
  ace.ace_performSelectionChange([rep.selStart[0],rep.selStart[1]-1], rep.selStart, false);
  ace.ace_performDocumentApplyAttributesToRange(rep.selStart, rep.selEnd, [["sketchSpaceIsImage", imageId]]);

  return imageId;
};

/* used on the client side only */
sketchSpace = new sketchSpaceInit();
