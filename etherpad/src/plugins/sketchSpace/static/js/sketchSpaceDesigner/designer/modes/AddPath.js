dojo.provide("sketchSpaceDesigner.designer.modes.AddPath");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");

dojo.declare("sketchSpaceDesigner.designer.modes.AddPath", [sketchSpaceDesigner.designer.modes.Zoom], {
  enable: function () {
    this.inherited(arguments);
    this.shape = undefined;
    this.smothenessFactor = 6;
    this.isClosed = false;
    this.isLine = false;
  },
  disable: function () {
    this.inherited(arguments);
    if (this.shape !== undefined) {
      this.shape.removeShape();
    }
  },
  getContainerShape: function () { return this.designer.surface_transform; },
  onKeyUp: function (event) {
    this.inherited(arguments);
    if (event.keyCode == 38 && !event.ctrlKey && !event.altKey && !event.shiftKey) {/* key=UP */
      this.smothenessFactor = Math.max(4, this.smothenessFactor + 3);
      this.redrawShape();
    } else if (event.keyCode == 40 && !event.ctrlKey && !event.altKey && !event.shiftKey) {/* key=DOWN */
      this.smothenessFactor = Math.max(4, this.smothenessFactor - 3);
      this.redrawShape();
    } else if (event.keyCode == 67 && !event.ctrlKey && !event.altKey && !event.shiftKey) { /* key=c */
      this.isClosed = !this.isClosed;
      this.redrawShape();
    } else if (event.keyCode == 76 && !event.ctrlKey && !event.altKey && !event.shiftKey) { /* key=l */
      this.isLine = !this.isLine;
      this.redrawShape();
    } else if (event.keyCode == 27) {
      this.designer.popMode();
    }
  },
  onMouseDown: function (event) {
    this.inherited(arguments);
    if (event.button == 0) {
      this.isLine = event.ctrlKey;
      this.isClosed = event.altKey;
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
  redrawShape: function () {
    var point;
    var prevPoint;

    var halfStep = Math.floor(this.smothenessFactor / 2);
    this.shape.setShape({path: ""});
    this.shape.setAbsoluteMode(true);

    this.shape.moveTo(this.points[0].x, this.points[0].y);

    if (this.isLine) {
      point = this.points[this.points.length - 1];
      this.shape.lineTo(point.x, point.y);
    } else {
      for (var i = this.smothenessFactor; i < this.points.length; i += this.smothenessFactor) {
	point = this.points[i];
	prevPoint = this.points[i - halfStep];
	this.shape.smoothCurveTo(prevPoint.x, prevPoint.y, point.x, point.y);
      }

      if (this.isClosed) {
        point = this.points[0];
      } else {
        point = this.points[this.points.length - 1];
      }
      prevPoint = this.points[Math.min(i - halfStep, this.points.length - 1)];
      this.shape.smoothCurveTo(prevPoint.x, prevPoint.y, point.x, point.y);
    }
    if (this.isClosed) {
      this.shape.closePath();
    }
  }
});
