dojo.provide("sketchSpaceDesigner.designer.modes.AddPath");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");

dojo.declare("sketchSpaceDesigner.designer.modes.AddPath", [sketchSpaceDesigner.designer.modes.Zoom], {
  enable: function () {
    this.inherited(arguments);
    this.shape = undefined;
    this.smothenessFactor = 6;
  },
  disable: function () {
    this.inherited(arguments);
    if (this.shape !== undefined) {
      this.shape.removeShape();
    }
  },
  getContainerShape: function () { return this.designer.surface_transform; },

  onMouseDown: function (event) {
    this.inherited(arguments);
    if (event.button == 0 && !event.ctrlKey && !event.altKey && !event.shiftKey) {
      this.shape = dojox.gfx.utils.deserialize(this.getContainerShape(), {shape:{type:"path", path:""}, stroke:this.designer.stroke, fill:this.designer.fill});
      this.points = [this.getCurrentMouse(event)];
      this.redrawShape();
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
      this.points.push(this.getCurrentMouse(event));
      this.redrawShape();
    }
  },
  onKeyUp: function (event) {
    this.inherited(arguments);
    if (event.keyCode == 27)
      this.designer.popMode();
  },
  redrawShape: function () {
    var halfStep = Math.floor(this.smothenessFactor / 2);
    this.shape.setShape({path: ""});
    this.shape.setAbsoluteMode(true);

    this.shape.moveTo(this.points[0].x, this.points[0].y);

    for (var i = this.smothenessFactor; i < this.points.length; i += this.smothenessFactor) {
      var point = this.points[i];
      var prevPoint = this.points[i - halfStep];
      this.shape.smoothCurveTo(prevPoint.x, prevPoint.y, point.x, point.y);
      // this.shape.qSmoothCurveTo(point.x, point.y);
    }

    var point = this.points[0];
    var prevPoint = this.points[Math.min(i - halfStep, this.points.length - 1)];
    this.shape.smoothCurveTo(prevPoint.x, prevPoint.y, point.x, point.y);
    // this.shape.qSmoothCurveTo(point.x, point.y);

    this.shape.closePath();
  }
});
