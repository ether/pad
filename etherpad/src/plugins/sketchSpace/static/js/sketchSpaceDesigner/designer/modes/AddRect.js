dojo.provide("sketchSpaceDesigner.designer.modes.AddRect");

dojo.require("sketchSpaceDesigner.designer.modes.Mode");

dojo.declare("sketchSpaceDesigner.designer.modes.AddRect", [sketchSpaceDesigner.designer.modes.Mode], {
  enable: function () {
    var mode = this;
    this.onMouseDownHandle = this.designer.surface.connect("onmousedown", this, function (event) { mode.onMouseDown(event); });
    this.onMouseUpHandle = this.designer.surface.connect("onmouseup", this, function (event) { mode.onMouseUp(event); });
    this.onMouseMoveHandle = this.designer.surface.connect("onmousemove", this, function (event) { mode.onMouseMove(event); });
    this.onKeyUpHandle = dojo.connect(document, "onkeyup", this, function (event) { mode.onKeyUp(event); });
    this.shape = undefined;
  },
  disable: function () {
    dojo.disconnect(this.onMouseDownHandle);
    dojo.disconnect(this.onMouseUpHandle);
    dojo.disconnect(this.onMouseMoveHandle);
    dojo.disconnect(this.onKeyUpHandle);
    if (this.shape !== undefined) {
      this.shape.removeShape();
    }
  },
  onMouseDown: function (event) {
    this.shape = dojox.gfx.utils.deserialize(this.designer.surface, {shape:{type:"rect", x:event.layerX, y:event.layerY, width:1, height:1}, stroke:this.designer.stroke, fill:this.designer.fill});
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
      var shapeData = this.shape.getShape();
      if (event.layerX >= this.shape.origX) {
        shapeData.x = this.shape.origX;
        shapeData.width = event.layerX - this.shape.origX;
      } else {
        shapeData.x = event.layerX;
        shapeData.width = this.shape.origX - event.layerX;
      }
      if (event.layerY >= this.shape.origY) {
        shapeData.y = this.shape.origY;
        shapeData.height = event.layerY - this.shape.origY;
      } else {
        shapeData.y = event.layerY;
        shapeData.height = this.shape.origY - event.layerY;
      }
      this.shape.setShape(shapeData);
      console.log(["MOVE", event]);
    }
  },
  onKeyUp: function (event) {
    if (event.keyCode == 27)
      this.designer.popMode();
  }
});
