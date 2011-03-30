dojo.provide("sketchSpaceDesigner.designer.bbox");

dojo.declare("sketchSpaceDesigner.designer.bbox.Bbox", [], {
  constructor: function (copyFrom) {
    if (copyFrom === undefined) copyFrom = {};
    this.x = copyFrom.x;
    this.y = copyFrom.y;
    this.width = copyFrom.width;
    this.height = copyFrom.height;
  },

  copy: function() {
    return new sketchSpaceDesigner.designer.bbox.Bbox(this);
  },

  isEmpty: function(bbox) {
    if (bbox === undefined) bbox = this;
    return bbox.x === undefined;
  },

  makeEmpty: function() { this.x = undefined; this.y = undefined; this.width = undefined; this.height = undefined; },

  add: function(bbox) { /* rename to union? */
    if (this.isEmpty(bbox)) {
      /* Do nothing */
    } else if (this.isEmpty()) {
      this.x = bbox.x;
      this.y = bbox.y;
      this.width = bbox.width;
      this.height = bbox.height;
    } else {
      var old = {x:this.x, y:this.y}

      this.x = Math.min(this.x, bbox.x);
      this.y = Math.min(this.y, bbox.y);

      this.width = Math.max(old.x + this.width, bbox.x + bbox.width) - this.x;
      this.height = Math.max(old.y + this.height, bbox.y + bbox.height) - this.y;
    }

    return this;
  },

  intersection: function(bbox) {
    if (this.isEmpty(bbox)) {
      this.makeEmpty();
    } else if (this.isEmpty()) {
      /* Do nothing */
    } else {
      var old = {x:this.x, y:this.y}

      this.x = Math.max(this.x, bbox.x);
      this.y = Math.max(this.y, bbox.y);

      this.width = Math.max(Math.min(old.x + this.width, bbox.x + bbox.width) - this.x, 0);
      this.height = Math.max(Math.min(old.y + this.height, bbox.y + bbox.height) - this.y, 0);
    }

    return this;
  },

  addPoint: function(point) {
    if (this.x === undefined) {
      this.x = point.x;
      this.y = point.y;
      this.width = 0;
      this.height = 0;
    } else {
      if (point.x < this.x) {
	this.width += this.x - point.x;
	this.x = point.x;
      } else if (point.x > this.x + this.width) {
	this.width = point.x - this.x;
      }
      if (point.y < this.y) {
	this.height += this.y - point.y;
	this.y = point.y;
      } else if (point.y > this.y + this.height) {
	this.height = point.y - this.y;
      }
    }
    return this;
  },

  addPoints: function(points) {
    var bbox = this;
    $.each(points, function (index, point) {
      bbox.addPoint(point);
    });
    return this;
  },

  transform: function (matrix) {
    if (!this.isEmpty()) {
      var x = [this.x, this.x + this.width];
      var y = [this.y, this.y + this.height];
      this.makeEmpty();
      this.addPoints($.map(
	[{x:x[0], y:y[0]},
	 {x:x[0], y:y[1]},
	 {x:x[1], y:y[0]},
	 {x:x[1], y:y[1]}],
	function (p) {
	  return dojox.gfx.matrix.multiplyPoint(matrix, p.x, p.y);
	}
      ));
    }
    return this;
  }
});
