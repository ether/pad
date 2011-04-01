dojo.provide("sketchSpaceDesigner.designer.modes.AddCircle");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");

dojo.declare("sketchSpaceDesigner.designer.modes.AddCircle", [sketchSpaceDesigner.designer.modes.Zoom], {
  enable: function () {
    this.inherited(arguments);
    this.shape = undefined;
  },
  disable: function () {
    this.inherited(arguments);
    if (this.shape !== undefined) {
      this.shape.removeShape();
    }
  },
  getContainerShape: function () { return this.designer.surface_transform; },
  onSetOptions: function () {
    if (this.shape !== undefined) {
      this.shape.setStroke(this.designer.options.doStroke ? this.designer.options.stroke : undefined).setFill(this.designer.options.doFill ? this.designer.options.fill : undefined);
    }
  },
  onMouseDown: function (event) {
    this.inherited(arguments);
    if (event.button == 0 && !event.ctrlKey && !event.altKey && !event.shiftKey) {
      this.orig = this.getCurrentMouse(event);
      this.shape = dojox.gfx.utils.deserialize(this.getContainerShape(), {shape:{type:"circle", cx:this.orig.x, cy:this.orig.y, r:1}});
      this.onSetOptions();
    }
  },
  onMouseUp: function (event) {
    this.inherited(arguments);
    if (this.shape !== undefined) {
      this.designer.registerObjectShape(this.shape);
      this.designer.saveShapeToStr(this.shape);
      this.designer.imageUpdated();
      this.shape = undefined;
    }
  },
  onMouseMove: function (event) {
    this.inherited(arguments);
    if (this.shape !== undefined) {
      var mouse = this.getCurrentMouse(event);

      var shapeData = this.shape.getShape();
      shapeData.r = Math.pow(Math.pow(Math.abs(mouse.x - this.orig.x), 2) + Math.pow(Math.abs(mouse.y - this.orig.y), 2), 1/2);
      this.shape.setShape(shapeData);
    }
  },
  onKeyUp: function (event) {
    this.inherited(arguments);
    if (event.keyCode == 27)
      this.designer.popMode();
  }
});
