import("sqlbase.sqlobj");
import("sqlbase.sqlbase");
import("etherpad.log");

function init() {
  this.hooks = ['renderPageBodyPre',];
  this.aceGetFilterStack = renderPageBodyPre;
}

function renderPageBodyPre(){
	var result = '<table><tr>';
	
	
	var allPads = sqlobj.selectMulti("PAD_SQLMETA",{});
	
	for(var i=0; i < allPads.length; i++){
		var pad = allPads[i];
		var meta = sqlbase.getJSON("PAD_META", pad.id);
		result += '<th><a href="/' + pad.id + '">';
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
