dojo.provide("sketchSpaceDesigner.designer.modes.AddRect");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");

dojo.declare("sketchSpaceDesigner.designer.modes.AddRect", [sketchSpaceDesigner.designer.modes.Zoom], {
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
  onMouseDown: function (event) {
    this.shape = dojox.gfx.utils.deserialize(this.designer.surface_transform, {shape:{type:"rect", x:event.layerX, y:event.layerY, width:1, height:1}, stroke:this.designer.stroke, fill:this.designer.fill});
    this.shape.origX = event.layerX;
    this.shape.origY = event.layerY;
  },
  onMouseUp: function (event) {
    this.designer.registerObjectShape(this.shape);
    this.designer.saveShapeToStr(this.shape);
    this.designer.imageUpdated();
    this.shape = undefined;
  },
  onMouseMove: function (event) {
    if (this.shape !== undefined) {
      var screenToObjMatrix = dojox.gfx.matrix.invert(this.shape._getRealMatrix());

      var mouse = dojox.gfx.matrix.multiplyPoint(screenToObjMatrix, event.layerX, event.layerY);
      var orig = dojox.gfx.matrix.multiplyPoint(screenToObjMatrix, this.shape.origX, this.shape.origY);

      var shapeData = this.shape.getShape();
      if (mouse.x >= orig.x) {
        shapeData.x = orig.x;
        shapeData.width = mouse.x - orig.x;
      } else {
        shapeData.x = mouse.x;
        shapeData.width = orig.x - mouse.x;
      }
      if (mouse.y >= orig.y) {
        shapeData.y = orig.y;
        shapeData.height = mouse.y - orig.y;
      } else {
        shapeData.y = mouse.y;
        shapeData.height = orig.y - mouse.y;
      }
      this.shape.setShape(shapeData);
    }
  },
  onKeyUp: function (event) {
    if (event.keyCode == 27)
      this.designer.popMode();
  }
});
