import("etherpad.utils.*");
import("etherpad.log");
import("etherpad.control.pad.pad_control");
import("sqlbase.sqlobj");
import("sqlbase.sqlbase");
import("etherpad.pad.exporthtml");
import("etherpad.pad.model");
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
	var pads = [];
	// seems like too many db queries - is there a selectMultiJSON command thing? getAllJSON? Does that support filter conditions?
	for(var i in matching_pads) pads.push(sqlbase.getJSON("PAD_META", matching_pads[i].id));
	
	var summary_pad_id = request.path.replace(/^\/specs\//,'').replace(/\/$/,'');
	
	var summary;
	model.accessPadGlobal(summary_pad_id, function(pad){
		if(pad.exists())			
			summary	= exporthtml.getPadHTML( pad );	
	}, 'r');
	 
	
	renderHtml('groupIndex.ejs',{	summary_pad_id:summary_pad_id,
									summary:summary,
									section:section,
									filter:filter, 
									pads:pads,
									groupBasedId:groupBasedId,
									shortName:shortName},'supportGroupUrls');
	return true;
}
function render_page(){
	var padId = request.path.toString().split("/specs/")[1].replace(/\//g,"-");
	return pad_control.render_pad(padId);
}
