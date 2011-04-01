/* KEYCODES:
SHIFT=16
CTRL=17
ALT=18
 */
dojo.provide("sketchSpaceDesigner.designer.modes.AddPath");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");
dojo.require("sketchSpaceDesigner.utils");

dojo.declare("sketchSpaceDesigner.designer.modes.Path", [], {
  constructor: function (mode) {
    this.mode = mode;
    this.sections = [];
    this.options = {};
    this.shape = dojox.gfx.utils.deserialize(mode.getContainerShape(), {shape:{type:"path", path:""}});
    this.setOptions(mode.designer.options);
  },
  setOptions: function (options) {
    sketchSpaceDesigner.utils.setObject(this.options, options);
    var section = this.getLastSection();
    if (section !== undefined)
      section.setOptions(options);
    this.renderToShape();
  },
  addSection: function () {
    this.sections.push(new sketchSpaceDesigner.designer.modes.PathSection(this));
    this.getLastSection().setOptions(this.options);
  },
  removeSection: function () {
    this.sections.pop();
  },
  getLastSection: function () {
    return this.sections[this.sections.length-1];
  },
  addPoint: function (p) {
    this.getLastSection().addPoint(p);
    this.renderToShape();
  },
  renderToShape: function () {
   this.shape.setShape({path: ""}).setStroke(this.options.doStroke ? this.options.stroke : undefined).setFill(this.options.doFill ? this.options.fill : undefined);

    this.shape.setAbsoluteMode(true);

    if (this.sections.length > 0 && this.sections[0].points.length > 0) {
      this.shape.moveTo(this.sections[0].points[0].x, this.sections[0].points[0].y);

      dojo.forEach(this.sections, function(section, i) {
	section.renderToShape(this.shape);
      });
    }

    if (this.options.isClosed) {
      this.shape.closePath();
    }
  }
});

dojo.declare("sketchSpaceDesigner.designer.modes.PathSection", [], {
  constructor: function (path) {
    this.path = path;
    this.options = {};
    this.setOptions(this.path.options);
    this.points = [];
  },
  setOptions: function (options) {
    sketchSpaceDesigner.utils.setObject(this.options, options);
  },
  addPoint: function (p) {
    this.points.push(p);
  },
  renderToShape: function () {
    if (this.points.length == 0) return;
    var point;
    var prevPoint;

    var halfStep = Math.max(1, Math.floor(this.options.smothenessFactor / 2));

    if (this.options.isLine) {
      point = this.points[this.points.length - 1];
      this.path.shape.lineTo(point.x, point.y);
    } else {
      var i;
 
      for (i = this.options.smothenessFactor; i < this.points.length; i += this.options.smothenessFactor) {
	point = this.points[i];
	prevPoint = this.points[i - halfStep];
	this.path.shape.smoothCurveTo(prevPoint.x, prevPoint.y, point.x, point.y);
      }

      if (this.options.isClosed) {
        point = this.points[0];
      } else {
        point = this.points[this.points.length - 1];
      }
      prevPoint = this.points[Math.min(i - halfStep, this.points.length - 1)];

      this.path.shape.smoothCurveTo(prevPoint.x, prevPoint.y, point.x, point.y);
    }
  }
});

dojo.declare("sketchSpaceDesigner.designer.modes.AddPath", [sketchSpaceDesigner.designer.modes.Zoom], {
  enable: function () {
    this.inherited(arguments);
    this.designer.setOptions({smothenessFactor: 6, isClosed: false, isLine: false}, true);
  },
  disable: function () {
    this.inherited(arguments);
    if (this.path !== undefined) {
      this.path.shape.removeShape();
    }
  },
  getContainerShape: function () { return this.designer.surface_transform; },
  onKeyDown: function (event) {
    this.inherited(arguments);
    if (event.keyCode == 17) { /* key=CTRL */
      this.designer.setOptions({isLine: true});
    } else if (event.keyCode == 18) { /* key=ALT */
      this.designer.setOptions({isClosed: true});
    }
  },
  onKeyUp: function (event) {
    this.inherited(arguments);
    if (event.keyCode == 38 && !event.ctrlKey && !event.altKey && !event.shiftKey) {/* key=UP */
      this.designer.setOptions({smothenessFactor: Math.max(4, this.designer.options.smothenessFactor + 3)});
    } else if (event.keyCode == 40 && !event.ctrlKey && !event.altKey && !event.shiftKey) {/* key=DOWN */
      this.designer.setOptions({smothenessFactor: Math.max(4, this.designer.options.smothenessFactor - 3)});
    } else if (event.keyCode == 67 && !event.ctrlKey && !event.altKey && !event.shiftKey) { /* key=c */
      this.designer.setOptions({isClosed: !this.designer.options.isClosed});
    } else if (event.keyCode == 76 && !event.ctrlKey && !event.altKey && !event.shiftKey) { /* key=l */
      this.designer.setOptions({isLine: !this.designer.options.isLine});
    } else if (event.keyCode == 17) { /* key=CTRL */
      this.designer.setOptions({isLine: false});
    } else if (event.keyCode == 18) { /* key=ALT */
      this.designer.setOptions({isClosed: false});
    } else if (event.keyCode == 13 && !event.ctrlKey && !event.altKey && !event.shiftKey) { /* key=ENTER */
      this.done();
    } else if (event.keyCode == 27) {
      this.designer.popMode();
    }
  },
  onSetOptions: function () {
    if (this.path !== undefined)
      this.path.setOptions(this.designer.options);
  },
  onMouseDown: function (event) {
    this.inherited(arguments);
    if (event.button == 0) {
      if (this.path === undefined) {
        this.path = new sketchSpaceDesigner.designer.modes.Path(this);
        this.path.addSection();
        this.path.addPoint(this.getCurrentMouse(event));
      }
    }
  },
  onMouseUp: function (event) {
    this.inherited(arguments);
    if (this.path !== undefined) {
      if (event.button == 2) {
        this.path.removeSection();
        this.done()
      } else {
        this.path.addSection();
        this.path.addPoint(this.getCurrentMouse(event));
      }
    }
  },
  onMouseMove: function (event) {
    this.inherited(arguments);
    if (this.path !== undefined) {
      this.path.addPoint(this.getCurrentMouse(event));
    }
  },
  done: function () {
    if (this.path !== undefined) {
      if (this.path.sections.length > 0) {
        this.designer.registerObjectShape(this.path.shape);
        this.designer.saveShapeToStr(this.path.shape);
        this.designer.imageUpdated();
      } else {
        this.path.shape.removeShape();
      }
      this.path = undefined;
    }
  },
});
