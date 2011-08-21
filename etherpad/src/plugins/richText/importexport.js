jimport("java.lang.System.out.println");
var spanStyle = ['color', 'backgroundColor', 'fontSize',
    'fontFamily', 'width', 'height'];

/**
 * conver fontSize to font-size
 */
function getCSSRuleName(name){
  return (name || "").replace(/([A-Z])/g,function(_, s){return '-' + s.toLowerCase()})
}

function isSpanStyle(name){
  for(var i = 0, len = spanStyle.length; i < len; i++){
    if (name == spanStyle[i]){
      return true;
    }
  }
  return false;
}


function exportInlineStyle(args){
  var result = {
    isAceObject    : false,
    extraOpenTags  : "",
    extraCloseTags : ""
  }  
  if (args && args.attributes) {
    var useSpan = false;
    var spanStyleList = {};
    for(var i = 0, len = args.attributes.length; i < len; i++) {
      attribName  = args.attributes[i].name;
      attribValue = args.attributes[i].value;
      if (isSpanStyle(attribName)){
        var cssRuleName = getCSSRuleName(attribName);   
        spanStyleList[cssRuleName] = attribValue;
        useSpan = true;
      } else {
        var attStr = "";
        switch (attribName){
          case "imgSrc":
              result.extraOpenTags += "<img src=\"" + attribValue + "\" />";
              break;
          case "aceObject":
              result.isAceObject = true;
              break;
          default:
              break;
        }
      }
    }
  }
  if (useSpan) {
    var openSpan = "<span style=\"";   
    for (var style in spanStyleList){
      openSpan += style + ":" + spanStyleList[style] + ";";
    }
    openSpan += "\">"
    result.extraOpenTags  += openSpan;
    result.extraCloseTags += "</span>";
  }
  return [result];
}
