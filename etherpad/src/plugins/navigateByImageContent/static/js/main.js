import("sqlbase.sqlobj");
import("sqlbase.sqlbase");
import("etherpad.log");

function init() {
  this.hooks = ['renderNavigation',];
  this.aceGetFilterStack = renderNavigation;
}

function renderNavigation(){
	var result = '<table><tr>';
	var query= {};
	var getPath = function(id){return id;};
	var match = request.path.match(/^((\/specs\/)([^\/]+\/)+)([^\/]+)$/);
	if(match){ 
		
		var section = request.path.replace(match[2], '').replace(match[4], '');
		var like = section.replace(/\//g,"-");
		log.info("\n\nlike query = " + like + "\n" + match + " \n");
		query = { id:['like', like + "%" ]  };
		getPath = function(id){return match[1] + id.replace(like,'');};
	}
	
	var allPads = sqlobj.selectMulti("PAD_SQLMETA",query);
	
	for(var i=0; i < allPads.length; i++){
		var pad = allPads[i];
		var meta = sqlbase.getJSON("PAD_META", pad.id);
		result += '<th><a href="' + getPath(pad.id) + '" title="'+pad.id+'">';
		if (meta && meta.images && meta.images.constructor && meta.images.constructor.toString().match(/Array/)) {
			
			result += '<img src="' + meta.images[0] + '" alt="' + pad.id + '" width="60px"/>';
		}
		else {
			result += pad.id;
		}	
		result += '</a></th>';
	}
	
	return result + '</tr></table>';
}
navigateByImageContent = new init();
