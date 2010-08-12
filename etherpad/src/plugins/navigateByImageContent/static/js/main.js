import("sqlbase.sqlobj");
import("sqlbase.sqlbase");
import("etherpad.log");
import("etherpad.utils.*");
function init() {
  this.hooks = ['renderNavigation',];
  this.aceGetFilterStack = renderNavigation;
}

function renderNavigation(){
	var result = '<table><tr>';
	var section;
	var query= {};
	var getPath = function(id){return id;};
	var match = request.path.match(/^((\/specs\/)([^\/]+\/)+)([^\/]+)$/);
	if(match){ 
		section = request.path.replace(match[2], '').replace(match[4], '');
		var like = section.replace(/\//g,"-");
		query = { id:['like', like + "%" ]  };
		getPath = function(id){return match[1] + id.replace(like,'');};
	}
	
	var allPads = sqlobj.selectMulti("PAD_SQLMETA",query);
	
	var pads = allPads.map(function(pad){
		var meta = sqlbase.getJSON("PAD_META", pad.id);
		var image_available =   (meta && meta.images && meta.images.constructor && meta.images.constructor.toString().match(/Array/));
		var src = image_available ? meta.images[0] : '/static/img/plugins/navigateByImageContent/placeholder.png'; 		
		var width = image_available ? 60 : 24;
		return {
			pad: pad,
			meta: meta,
			image_available:image_available,
			src:src,
			width:width,
			path:getPath(pad.id)
		};
				 
	});
	
	return renderTemplateAsString('imageNavigation.ejs',{pads:pads, section:section}, 'navigateByImageContent');
}
navigateByImageContent = new init();
