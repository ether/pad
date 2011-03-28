dojo.provide("sketchSpaceDesigner.designer");

dojo.require("sketchSpaceDesigner.designer.modes");
dojo.require("sketchSpaceDesigner.designer.bbox");
dojo.require("sketchSpaceDesigner.designer.selection");
dojo.require("dojox.gfx");
dojo.require("dojox.gfx.move");
dojo.require("dojox.gfx.utils");
dojo.require("dojox.gfx.matrix");
dojo.require("dojox.uuid.generateRandomUuid");

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

  saveShapeToStr: function(shape) {
    var parent = null;
    if (shape.parent.objId != undefined)
      parent = shape.parent.objId;

    shape.strRepr = dojo.toJson({parent:parent, shape:dojox.gfx.utils.serialize(shape)});
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
      if (shape.objId === undefined) return;
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
    var shape = dojox.gfx.utils.deserialize(this.editorGetShapeByObjId(shapeDescription.parent), shapeDescription.shape);
    this.registerObjectShape(shape);
    this.saveShapeToStr(shape);
  },

  addRect: function() {
    this.pushMode(new sketchSpaceDesigner.designer.modes.AddRect());
  },

  addCircle: function() {
    this.pushMode(new sketchSpaceDesigner.designer.modes.AddCircle());
  },


});

dojo.addOnLoad(function (){
  sketchSpace.editorArea = new sketchSpaceDesigner.designer.Designer(dojo.byId("sketchSpaceDebug"), 300, 300);
  dojo.connect(sketchSpace.editorArea, "imageUpdated", sketchSpace, sketchSpace.updatePadFromImage);
});
