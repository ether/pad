dojo.provide("sketchSpaceDesigner.designer");

dojo.require("sketchSpaceDesigner.designer.modes");
dojo.require("sketchSpaceDesigner.designer.bbox");
dojo.require("sketchSpaceDesigner.designer.selection");
dojo.require("dojox.gfx");
dojo.require("dojox.gfx.move");
dojo.require("dojox.gfx.utils");
dojo.require("dojox.gfx.matrix");
dojo.require("dojox.uuid.generateRandomUuid");
dojo.require("dojo.parser");
//dojo.require("dijit.popup");
dojo.require("dojox.widget.ColorPicker");

dojo.declare("sketchSpaceDesigner.designer.Designer", [], {
  constructor: function (container, width, height) {
    this.container = container;
    this.surface = dojox.gfx.createSurface(this.container, width, height);
    this.surface_transform = this.surface.createGroup();
    this.surface_size = {width: width, height: height};

    this.images = {};
    this.currentImage = undefined;
    this.selection = new sketchSpaceDesigner.designer.selection.Selection(this);

    dojo.connect(this.container, "ondragstart",   dojo, "stopEvent");
    dojo.connect(this.container, "onselectstart", dojo, "stopEvent");

    this.modeStack = [];
    this.pushMode(new sketchSpaceDesigner.designer.modes.Select());

    this.stroke = {"type":"stroke","color":{"r":0,"g":255,"b":0,"a":1},"style":"solid","width":2,"cap":"butt","join":4};
    this.fill = {"r":255,"g":0,"b":0,"a":1};
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

  deserializeShape: function(parent, shape) {
    if (shape.extType == "zimage") {
      return this.createImage(parent, shape.imageName).setTransform(shape.transform);
    } else {
      return dojox.gfx.utils.deserialize(parent, shape);
    }
  },

  serializeShape: function(shape) {
    /* FIXME: Remove "children" from serialized groups */
    if (shape.extType == "zimage") {
      return {extType: "zimage", imageName: shape.imageName, transform:shape.getTransform()};
    } else {
      return dojox.gfx.utils.serialize(shape);
    }
  },

  saveShapeToStr: function(shape) {
    var parent = null;
    if (shape.parent.objId != undefined)
      parent = shape.parent.objId;

    shape.strRepr = dojo.toJson({parent:parent, shape:this.serializeShape(shape)});
    this.imageUpdated();
  },

  /* Use this to listen for changes */
  imageUpdated: function () {},

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
    this.getCurrentMode().enableShape(shape);
  },

  unregisterObjectShape: function(shape) {
    this.getCurrentMode().disableShape(shape);
  },

  editorShapeRemove: function(shape) {
    this.unregisterObjectShape(shape);
    shape.removeShape();
    this.imageUpdated();
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

  createImage: function(parent, imageName) {
    var designer = this;

    var image = parent.createGroup();
    image.extType = "zimage";
    image.currentDisplay = image.createImage();
    image.imageName = imageName;
    image.updateDisplay = function () {
      var image = this;
      if (this.pointSize === undefined) {
	dojo.xhrGet({
	  url: "/ep/imageConvert/" + this.imageName + "?action=getSize",
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
	  src: "/ep/imageConvert/" + this.imageName + "?p=0&x=" + displayBboxOnObj.x + "&y=" + displayBboxOnObj.y + "&w=" + displayBboxOnObj.width + "&h=" + displayBboxOnObj.height + "&pw=" + displayBboxOnScreen.width + "&ph=" + displayBboxOnScreen.height
        };

	var oldShape = this.currentDisplay.getShape()

	if (oldShape.src != newShape.src) {
	  var lastDisplay = this.currentDisplay;
	  this.currentDisplay = image.createImage();
  	  this.currentDisplay.setShape(newShape);
	  this.currentDisplayTimout = window.setTimeout(function () {
            lastDisplay.removeShape();
	  }, 500);
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
    image.zoomHandle = dojo.connect(designer.surface_transform, "setTransform", image, image.updateDisplayLazy);
    image.updateHandle = dojo.connect(designer, "imageUpdated", image, image.updateDisplayLazy);

    return image;
  },

  addImg: function(imageName) {
    var shape = this.createImage(this.surface_transform, imageName);
    this.registerObjectShape(shape);
    this.saveShapeToStr(shape);
    this.imageUpdated();
  },

  /* refactor out this code and put it somewhere else... */
  foregroundColorPickerPopup: function() {
    this.foregroundColorPicker.popup();
  },

  backgroundColorPickerPopup: function() {
    this.backgroundColorPicker.popup();
  },

});

dojo.declare("sketchSpaceDesigner.designer.ColorPickerPopup", [dojox.widget.ColorPicker], {
  create: function () {
    this.inherited(arguments);
    dijit.popup.moveOffScreen(this.domNode);
  },
  popup: function () {
    var widget = this;
    dijit.popup.open({
      parent: null,
      popup: widget,
      around: widget.popupFor,
      orient: {'BR':'TR', 'BL':'TL', 'TR':'BR', 'TL':'BL'},
      onExecute: function(){
	dijit.popup.close(widget);
        widget.setColor(widget.attr("value"));
      },
      onCancel: function(){ dijit.popup.close(widget); },
      onClose: function(){}
    });
    this.focus();
  },
  setColor: function(colorHex) {
    this.inherited(arguments);
    dojo.style(this.popupFor, "background", colorHex);
  },
  onBlur: function () {
    this.inherited(arguments);
    this.onCancel();
  },
  onCancel: function () {},
});

dojo.addOnLoad(function (){
  sketchSpace.editorArea = new sketchSpaceDesigner.designer.Designer(dojo.byId("sketchSpaceDebug"), 300, 300);
  dojo.connect(sketchSpace.editorArea, "imageUpdated", sketchSpace, sketchSpace.updatePadFromImage);

  sketchSpace.editorArea.foregroundColorPicker = new sketchSpaceDesigner.designer.ColorPickerPopup({popupFor: dojo.byId("foregroundColorPicker")});
  dojo.connect(sketchSpace.editorArea.foregroundColorPicker, "setColor", sketchSpace.editorArea, function (colorHex) { this.stroke.color = dojo.colorFromHex(colorHex); });
  sketchSpace.editorArea.backgroundColorPicker = new sketchSpaceDesigner.designer.ColorPickerPopup({popupFor: dojo.byId("backgroundColorPicker")});
  dojo.connect(sketchSpace.editorArea.backgroundColorPicker, "setColor", sketchSpace.editorArea, function (colorHex) { this.fill = dojo.colorFromHex(colorHex); });

  $(function(){  
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
  });

});
