dojo.require("dojox.gfx");
dojo.require("dojox.gfx.move");
dojo.require("dojox.gfx.utils");

function getRand(from, to){
	return Math.random() * (to - from) + from;
}

var skew_stat_factor = 15;

function getRandSkewed(from, to){
	// let skew stats to smaller values
	var seed = 0;
	for(var i = 0; i < skew_stat_factor; ++i){
		seed += Math.random();
	}
	seed = 2 * Math.abs(seed / skew_stat_factor - 0.5);
	return seed * (to - from) + from;
}

function randColor(alpha){
	var red   = Math.floor(getRand(0, 255)),
		green = Math.floor(getRand(0, 255)),
		blue  = Math.floor(getRand(0, 255)),
		opacity = alpha ? getRand(0.1, 1) : 1;
	return [red, green, blue, opacity];
}


function initGfx(){
  openingDesign.editorArea = {};
  openingDesign.editorArea.container = dojo.byId("openingDesignDebug");
  openingDesign.editorArea.surface = dojox.gfx.createSurface(openingDesign.editorArea.container, 300, 300);
  openingDesign.editorArea.surface_size = {width: 300, height: 300};

  // // cancel text selection and text dragging
  dojo.connect(openingDesign.editorArea.container, "ondragstart",   dojo, "stopEvent");
  dojo.connect(openingDesign.editorArea.container, "onselectstart", dojo, "stopEvent");


   var editorArea = top.openingDesign.editorArea;
   var minR = 10, maxR = editorArea.surface_size.width / 3;
   for(var j = 0; j < 5; ++j){
	   var r = getRandSkewed(minR, maxR),
		   cx = getRand(r, editorArea.surface_size.width  - r),
		   cy = getRand(r, editorArea.surface_size.height - r),
		   shape = editorArea.surface.createCircle({cx: cx, cy: cy, r: r})
			   .setFill(randColor(true))
			   .setStroke({color: randColor(true), width: getRand(0, 3)})
			   ;
	   new dojox.gfx.Moveable(shape);
   }
}

dojo.addOnLoad(initGfx);
