import("etherpad.utils.*");
import("etherpad.log");
import("etherpad.control.pad.pad_control");
import("sqlbase.sqlobj");
function onRequest() {
	var section = request.path.toString().split("/specs/")[1];
	var filter = section.replace(/\//g,"-");
	function shortName(pad_id){
		return pad_id.replace(filter, '');
	}
	function groupBasedId(pad_id){
		return request.path + shortName(pad_id);
	}
	
	var matching_pads = sqlobj.selectMulti("PAD_SQLMETA",{id:['like', filter+'%']});
	renderHtml('groupIndex.ejs',{section:section,
									filter:filter, 
									pads:matching_pads,
									groupBasedId:groupBasedId,
									shortName:shortName},'supportGroupUrls');
	return true;
}
function render_page(){
	var padId = request.path.toString().split("/specs/")[1].replace(/\//g,"-");
	return pad_control.render_pad(padId);
}
