dojo.provide("sketchSpaceDesigner.designer.bbox");

dojo.declare("sketchSpaceDesigner.designer.bbox.Bbox", [], {
  construct: function (copyFrom) {
    if (copyFrom === undefined) copyFrom = {};
    this.x = copyFrom.x;
    this.y = copyFrom.y;
    this.width = copyFrom.width;
    this.height = copyFrom.height;
  },

  copy: function() {
    return new sketchSpaceDesigner.designer.bbox.Bbox(this);
  },

  add: function(bbox) {
    var old = {x:this.x, y:this.y}
    
    this.x = min(this.x, bbox.x);
    this.y = min(this.y, bbox.y);

    this.width = max(olx.x + this.width, bbox.x + bbox.width) - this.x;
    this.height = max(old.y + this.height, bbox.y + bbox.height) - this.y;

    return this;
  },

  addPoints: function(points) {
    var bbox = this;
    $.each(points, function (index, point) {
      if (bbox.x === undefined) {
        bbox.x = point.x;
        bbox.y = point.y;
        bbox.width = 0;
	bbox.height = 0;
      } else {
	if (point.x < bbox.x) {
	  bbox.width += bbox.x - point.x;
	  bbox.x = point.x;
	} else if (point.x > bbox.x + bbox.width) {
	  bbox.width = point.x - bbox.x;
	}
	if (point.y < bbox.y) {
	  bbox.height += bbox.y - point.y;
	  bbox.y = point.y;
	} else if (point.y > bbox.y + bbox.height) {
	  bbox.height = point.y - bbox.y;
	}
      }
    });
    return this;
  }
});
