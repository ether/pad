dojo.provide("sketchSpaceDesigner.designer");

dojo.require("sketchSpaceDesigner.utils");
dojo.require("sketchSpaceDesigner.designer.modes");
dojo.require("sketchSpaceDesigner.designer.bbox");
dojo.require("sketchSpaceDesigner.designer.selection");
dojo.require("sketchSpaceDesigner.designer.widgets");
dojo.require("dojox.gfx");
dojo.require("dojox.gfx.move");
dojo.require("dojox.gfx.utils");
dojo.require("dojox.gfx.matrix");
dojo.require("dojox.uuid.generateRandomUuid");
dojo.require("dojo.parser");
dojo.require("dojox.layout.TableContainer");
dojo.require("dijit.layout.ContentPane");

dojo.declare("sketchSpaceDesigner.designer.Designer", [], {
 constructor: function (container, userId) {
    this.container = container;
    
    this.surface_size = {width: $(container).width(), height: $(container).height()};
    this.userId = userId;

    this.surface = dojox.gfx.createSurface(this.container, this.surface_size.width, this.surface_size.height);
    this.surface_transform = this.surface.createGroup();
    
    this.viewUpdatedHandle = dojo.connect(this.surface_transform, "setTransform", this, function () { this.viewUpdated(); });

    this.images = {};
    this.currentImage = undefined;
    this.selection = new sketchSpaceDesigner.designer.selection.Selection(this);

    dojo.connect(this.container, "ondragstart",   dojo, "stopEvent");
    dojo.connect(this.container, "onselectstart", dojo, "stopEvent");

    this.modeStack = [];
    this.pushMode(new sketchSpaceDesigner.designer.modes.Select());

    this.options = {};
    this.setOptions({
      doStroke: true,
      doFill: true,
      stroke: {"type":"stroke","color":{"r":0,"g":255,"b":0,"a":1},"style":"solid","width":2,"cap":"butt","join":4},
      fill: {"r":255,"g":0,"b":0,"a":1},
      showAuthorshipColors: false,
    });
  },

  setOptions: function (options, onlyDefault) {
    sketchSpaceDesigner.utils.setObject(this.options, options, onlyDefault);
  },

  pushMode: function (mode) {
    if (this.modeStack.length > 0)
      this.getCurrentMode().disable();
    mode.designer = this;
    this.modeStack.push(mode);
    this.getCurrentMode().enable();
  },

  popMode: function () {
    this.getCurrentMode().disable();
    this.modeStack.pop();
    if (this.modeStack.length > 0)
      this.getCurrentMode().enable();
  },

  getCurrentMode: function () {
    return this.modeStack[this.modeStack.length - 1];
  },

  /* This function should really be somewhere else... */
  getUserColor: function (userId) {
    var palette = pad.getColorPalette();
    var userData;

    $.each(pad.collabClient.getConnectedUsers(), function () {
      if (this.userId == userId)
	userData = this;
    });

    if (userData === undefined) {
      userData = clientVars.collab_client_vars.historicalAuthorData[userId];
    }

    if (userData === undefined) {
      return;
    }

    return palette[userData.colorId]
  },

  deserializeShape: function(parent, description) {
    var shape;
    if (description.extType == "zimage") {
      shape = this.createImage(parent, description.imageName, description.page);
      if (description.transform !== undefined)
        shape.setTransform(description.transform);
    } else {
      shape = dojox.gfx.utils.deserialize(parent, description);
    }

    shape.userId = description.userId;
    shape.realColor = {fill: shape.getFill(),
		       stroke: shape.getStroke()};

    this.updateShapeAuthorshipColor(shape);

    return shape;
  },

  serializeShape: function(shape) {
    /* FIXME: Remove "children" from serialized groups */

    var description;

    if (shape.extType == "zimage") {
      description = {extType: "zimage", imageName: shape.imageName, page:shape.page, transform:shape.getTransform()};
    } else {
      description = dojox.gfx.utils.serialize(shape);
    }

    description.fill = shape.realColor.fill;
    description.stroke = shape.realColor.stroke;

    description.userId = shape.userId;
    return description;
  },

  saveShapeToStr: function(shape) {
    var parent = null;
    if (shape.parent.objId != undefined)
      parent = shape.parent.objId;

    shape.strRepr = dojo.toJson({parent:parent, shape:this.serializeShape(shape), userId:shape.userId});
    this.imageUpdatedByUs();
  },

  updateShapeAuthorshipColor: function (shape) {
    if (this.options.showAuthorshipColors) {
      var userColor = dojox.color.fromHex(this.getUserColor(shape.userId || this.userId));

      shape.setFill(shape.realColor.fill === undefined ? undefined : userColor);

      var stroke;
      if (shape.realColor.stroke !== undefined) {
	stroke = {};
	sketchSpaceDesigner.utils.setObject(stroke, shape.realColor.stroke);
	var strokeColor = dojox.color.fromArray(userColor.toRgba());
	strokeColor.r = Math.round(strokeColor.r / 2);
	strokeColor.g = Math.round(strokeColor.g / 2);
	strokeColor.b = Math.round(strokeColor.b / 2);
	stroke.color = strokeColor;
      }
      shape.setStroke(stroke);
    } else {
      shape.setFill(shape.realColor.fill);
      shape.setStroke(shape.realColor.stroke);
    }
  },

  setShapeFillAndStroke: function (shape, options) {
    shape.realColor = {fill: options.fill, stroke: options.stroke};

    this.updateShapeAuthorshipColor(shape);
  },

  /* Use this to listen for changes */
  imageUpdatedByUs: function () { this.imageUpdated(); },
  imageUpdatedByOthers: function () { this.imageUpdated(); },

  imageUpdated: function () { this.viewUpdated(); },

  viewUpdated: function () { },

  selectImage: function (imageId) {
    this.currentImage = imageId;
  },

  editorGetShapeByObjId: function(objId) {
    var designer = this;
    if (objId == null) return this.surface_transform;
    var res = undefined;
    dojox.gfx.utils.forEach(this.surface_transform, function (shape) {
      if (shape === designer.surface_transform) return;
      if (shape.objId == objId) res = shape;
    });
    return res;
  },

  forEachObjectShape: function(fn) {
    dojox.gfx.utils.forEach(this.surface_transform, function (shape) {
      if (shape === undefined || shape.objId === undefined) return;
      return fn(shape);
    });
  },

  registerObjectShape: function(shape) {
    if (shape.objId === undefined) {
      shape.objId = dojox.uuid.generateRandomUuid();
    }
    if (shape.userId === undefined) {
      shape.userId = this.userId;
    }
    this.getCurrentMode().enableShape(shape);
  },

  unregisterObjectShape: function(shape) {
    this.getCurrentMode().disableShape(shape);
  },

  editorShapeRemove: function(shape) {
    this.unregisterObjectShape(shape);
    shape.removeShape();
    this.imageUpdatedByUs();
  },

  editorAddShape: function(shapeDescription) {
    var shape = this.deserializeShape(this.editorGetShapeByObjId(shapeDescription.parent), shapeDescription.shape);
    this.registerObjectShape(shape);
    this.saveShapeToStr(shape);
  },

  addRect: function() {
    this.pushMode(new sketchSpaceDesigner.designer.modes.AddRect());
  },

  addCircle: function() {
    this.pushMode(new sketchSpaceDesigner.designer.modes.AddCircle());
  },

  addPath: function() {
    this.pushMode(new sketchSpaceDesigner.designer.modes.AddPath());
  },

  createImage: function(parent, imageName, page) {
    var designer = this;

    var image = parent.createGroup();
    image.extType = "zimage";
    image.currentDisplay = image.createImage();
    image.imageName = imageName;
    image.page = page ? page : 0;
    image.updateDisplay = function () {
      var image = this;
      if (this.pointSize === undefined) {
	dojo.xhrGet({
	  url: "/ep/imageConvert/" + this.imageName + "?action=getSize&p=" + image.page,
	  handleAs: "json",
	  load: function(data){
	    image.pointSize = data;
	    image.updateDisplay();
	  }
	});
      } else {
	var objToScreenMatrix = this._getRealMatrix();
	var screenToObjMatrix = dojox.gfx.matrix.invert(objToScreenMatrix);

	var screenBboxOnObj = new sketchSpaceDesigner.designer.bbox.Bbox({x: 0, y: 0, width:designer.surface_size.width, height:designer.surface_size.height}).transform(screenToObjMatrix);
	var objBboxOnObj = new sketchSpaceDesigner.designer.bbox.Bbox({x: 0, y: 0, width:this.pointSize.w, height:this.pointSize.h});

	var displayBboxOnObj = objBboxOnObj.copy().intersection(screenBboxOnObj).powround({x:2, y:2}, {x:8, y:8});
	var displayBboxOnScreen = displayBboxOnObj.copy().transform(objToScreenMatrix).powroundSize({x:2, y:2}, {x:8, y:8});

	//console.log("zoom: " + displayBboxOnObj.toString() + " @ " + displayBboxOnScreen.width + ":" + displayBboxOnScreen.height);

	var newShape = {
	  x:displayBboxOnObj.x,
	  y:displayBboxOnObj.y,
	  width:displayBboxOnObj.width,
	  height:displayBboxOnObj.height,
	  src: "/ep/imageConvert/" + this.imageName + "?p=" + image.page + "&x=" + displayBboxOnObj.x + "&y=" + displayBboxOnObj.y + "&w=" + displayBboxOnObj.width + "&h=" + displayBboxOnObj.height + "&pw=" + displayBboxOnScreen.width + "&ph=" + displayBboxOnScreen.height
        };

	var oldShape = this.currentDisplay.getShape()

	if (oldShape.src != newShape.src) {

	  image.newShape = newShape;

	  // Preload the image to the cache...
	  dojo.xhrGet({
	    url: newShape.src,
	    load: function(data){
	      /* Now when the image is in the cache, "load" the image */
	      /* We've already zoomed more, forget about it... */
 	      if (image.newShape != newShape) return;
	      var lastDisplay = this.currentDisplay;
	      image.currentDisplay = image.createImage();
	      image.currentDisplay.setShape(newShape);
	      if (lastDisplay) lastDisplay.removeShape();
	    }
	  });
        }
      }
    }
    image.updateDisplayLazy = function () {
      if (this.updateDisplayTimout !== undefined) return;
      var image = this;
      this.updateDisplayTimout = window.setTimeout(function () {
	image.updateDisplay();
	image.updateDisplayTimout = undefined;
      }, 500);
    }
    image.getTransformedBoundingBox = function () {
      var objToScreenMatrix = this._getRealMatrix();
      return new sketchSpaceDesigner.designer.bbox.Bbox({x: 0, y: 0, width:this.pointSize.w, height:this.pointSize.h}).transform(objToScreenMatrix).corners();
    }

    image.updateDisplay();
    image.updateHandle = dojo.connect(designer, "viewUpdated", image, image.updateDisplayLazy);

    return image;
  },

  addImg: function(imageName) {
    var shape = this.createImage(this.surface_transform, imageName);
    this.registerObjectShape(shape);
    this.saveShapeToStr(shape);
  },

  /* refactor out this code and put it somewhere else... */
  foregroundColorPickerPopup: function() {
    this.foregroundColorPicker.popup();
  },

  backgroundColorPickerPopup: function() {
    this.backgroundColorPicker.popup();
  },

});

dojo.declare("sketchSpaceDesigner.designer.DesignerUI", [dijit._Widget, dijit._Templated], {
  widgetsInTemplate: true,
  templateString: '<div>' +
                  '  <div id="sketchSpaceEditor" dojoAttachPoint="editorArea"></div>' +
                  '  <div id="sketchSpaceOptions" dojoType="dojox.layout.TableContainer" dojoAttachPoint="options" cols="1" showLabels="true">' +
                  '    <div dojoType="dijit.layout.ContentPane" title="Option">Value</div>' +
                  '  </div>' +
                  '</div>',
  startup: function () {
    this.inherited(arguments);

    this.editor = new sketchSpaceDesigner.designer.Designer(this.editorArea, this.attr("userId"));
    this.editor.ui = this;
  }
});


dojo.addOnLoad(function (){
  sketchSpace.editorUi = sketchSpaceDesigner.designer.DesignerUI({userId: pad.getUserId()}, dojo.byId("sketchSpaceEditorUI"));
  sketchSpace.editorUi.startup();

  /* For backwards compatibility for now */
  sketchSpace.editorArea = sketchSpace.editorUi.editor;

  dojo.connect(sketchSpace.editorArea, "imageUpdatedByUs", sketchSpace, sketchSpace.updatePadFromImage);


  sketchSpace.editorArea.foregroundColorPicker = new sketchSpaceDesigner.designer.widgets.ColorPickerPopup({popupFor: dojo.byId("foregroundColorPicker")});
  dojo.connect(sketchSpace.editorArea.foregroundColorPicker, "setColor", sketchSpace.editorArea, function (colorHex) { this.setOptions({stroke:{color:dojo.colorFromHex(colorHex)}}); });
  sketchSpace.editorArea.backgroundColorPicker = new sketchSpaceDesigner.designer.widgets.ColorPickerPopup({popupFor: dojo.byId("backgroundColorPicker")});
  dojo.connect(sketchSpace.editorArea.backgroundColorPicker, "setColor", sketchSpace.editorArea, function (colorHex) { this.setOptions({fill:dojo.colorFromHex(colorHex)}); });

  var info = {  
    action: '/ep/fileUpload/',
    name: 'uploadfile',  
    onSubmit: function(file, ext){
    //console.log('Starting...');
    },  
    onComplete: function(file, response){
      var path = eval(response)[0].split("/");
      sketchSpace.editorArea.addImg(path[path.length-1]);
    }
  }
  new AjaxUpload($('#sketchSpaceAddImage'), info);  
  new AjaxUpload($('#sketchSpaceAddImage img'), info);

  var info = {  
    action: '/ep/fileUpload/',
    name: 'uploadfile',  
    onSubmit: function(file, ext){
    //console.log('Starting...');
    },  
    onComplete: function(file, response){
      var path = eval(response)[0].split("/");
      var filename = path[path.length-1];
     
      dojo.xhrGet({
	url: "/ep/imageConvert/" + filename + "?action=getPages",
	handleAs: "json",
	load: function(data){
          padeditor.ace.callWithAce(function (ace) {
  	    for (var page = 0; page < data.pages; page++) {

	      var imageId = sketchSpace.ace_insertImage(ace);
	      var rep = ace.ace_getRep();
	      ace.ace_performDocumentApplyAttributesToRange(rep.selStart, rep.selEnd, [["sketchSpaceImageObject:" + dojox.uuid.generateRandomUuid(), escape(dojo.toJson({parent:null, shape: {extType: "zimage", imageName: filename, page:page}}))]]);
	      ace.ace_performSelectionChange(rep.selEnd, rep.selEnd, false);

	    }
	  }, "sketchSpace", true)
	}
      });

    }
  }

  new AjaxUpload($('#sketchSpaceAddPdfImage'), info);  
  new AjaxUpload($('#sketchSpaceAddPdfImage img'), info);

});
