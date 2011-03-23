dojo.provide("sketchSpaceDesigner.designer");

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
    this.surface_size = {width: width, height: height};

    this.images = {};
    this.currentImage = undefined;
    this.selection = new sketchSpaceDesigner.designer.selection.Selection(this);

    dojo.connect(this.container, "ondragstart",   dojo, "stopEvent");
    dojo.connect(this.container, "onselectstart", dojo, "stopEvent");
  },

  saveShapeToStr: function(shape) {
    var parent = null;
    if (shape.parent.objId != undefined)
      parent = shape.parent.objId;

    shape.strRepr = dojo.toJson({parent:parent, shape:dojox.gfx.utils.serialize(shape)});
  },

  /* Use this to listen for changes */
  imageUpdated: function () {},

  editorGetShapeByObjId: function(objId) {
    var designer = this;
    if (objId == null) return this.surface;
    var res = undefined;
    dojox.gfx.utils.forEach(this.surface, function (shape) {
      if (shape === designer.surface) return;
      if (shape.objId == objId) res = shape;
    });
    return res;
  },

  registerObjectShape: function(shape) {
    var designer = this;
    shape.moveable = new dojox.gfx.Moveable(shape);
    shape.shapeMovedSignalHandle = dojo.connect(shape.moveable, "onMoveStop", this, this.editorCallbackShapeMoved);
    shape.clickSignalHandle = shape.connect("onclick", shape, function (event) { designer.editorCallbackShapeClick(this, event); });
  },

  editorCallbackShapeMoved: function(mover) {
    this.saveShapeToStr(mover.host.shape);
    this.imageUpdated();
  },

  editorCallbackShapeClick: function(shape, event) {
    this.selection.editorShapeToggleSelection(shape, !event.ctrlKey);
  },

  editorShapeRemove: function(shape) {
    shape.removeShape();
    this.imageUpdated();
  },

  editorAddShape: function(shapeDescription) {
    var shape = dojox.gfx.utils.deserialize(this.editorGetShapeByObjId(shapeDescription.parent), shapeDescription.shape);
    shape.objId = dojox.uuid.generateRandomUuid();
    this.registerObjectShape(shape);
    this.saveShapeToStr(shape);
    this.imageUpdated();
  },

  editorAddCircle: function() {
    this.editorAddShape({parent:null,shape:{"shape":{"type":"circle","cx":100,"cy":100,"r":50},"stroke":{"type":"stroke","color":{"r":0,"g":255,"b":0,"a":1},"style":"solid","width":2,"cap":"butt","join":4},"fill":{"r":255,"g":0,"b":0,"a":1}}});
  },


});

dojo.addOnLoad(function (){
  sketchSpace.editorArea = new sketchSpaceDesigner.designer.Designer(dojo.byId("sketchSpaceDebug"), 300, 300);
  dojo.connect(sketchSpace.editorArea, "imageUpdated", sketchSpace, sketchSpace.updatePadFromImage);
});
