dojo.provide("sketchSpaceDesigner.designer.modes.AddCircle");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");

dojo.declare("sketchSpaceDesigner.designer.modes.AddCircle", [sketchSpaceDesigner.designer.modes.Zoom], {
  enable: function () {
    this.inherited(arguments);
    var mode = this;
    this.onMouseDownHandle = this.designer.surface.connect("onmousedown", this, function (event) { mode.onMouseDown(event); });
    this.onMouseUpHandle = this.designer.surface.connect("onmouseup", this, function (event) { mode.onMouseUp(event); });
    this.onMouseMoveHandle = this.designer.surface.connect("onmousemove", this, function (event) { mode.onMouseMove(event); });
    this.onKeyUpHandle = dojo.connect(document, "onkeyup", this, function (event) { mode.onKeyUp(event); });
    this.shape = undefined;
  },
  disable: function () {
    this.inherited(arguments);
    dojo.disconnect(this.onMouseDownHandle);
    dojo.disconnect(this.onMouseUpHandle);
    dojo.disconnect(this.onMouseMoveHandle);
    dojo.disconnect(this.onKeyUpHandle);
    if (this.shape !== undefined) {
      this.shape.removeShape();
    }
  },
  getContainerShape: function () { return this.designer.surface_transform; },

  onMouseDown: function (event) {
    this.orig = this.getCurrentMouse(event);
    this.shape = dojox.gfx.utils.deserialize(this.getContainerShape(), {shape:{type:"circle", cx:this.orig.x, cy:this.orig.y, r:1}, stroke:this.designer.stroke, fill:this.designer.fill});
  },
  onMouseUp: function (event) {
    this.designer.registerObjectShape(this.shape);
    this.designer.saveShapeToStr(this.shape);
    this.designer.imageUpdated();
    this.shape = undefined;
  },
  onMouseMove: function (event) {
    if (this.shape !== undefined) {
      var mouse = this.getCurrentMouse(event);

      var shapeData = this.shape.getShape();
      shapeData.r = Math.pow(Math.pow(Math.abs(mouse.x - this.orig.x), 2) + Math.pow(Math.abs(mouse.y - this.orig.y), 2), 1/2);
      this.shape.setShape(shapeData);
    }
  },
  onKeyUp: function (event) {
    if (event.keyCode == 27)
      this.designer.popMode();
  }
});
