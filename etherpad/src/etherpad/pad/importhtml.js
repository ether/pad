/**
 * Copyright 2009 Google Inc.
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 * 
 *      http://www.apache.org/licenses/LICENSE-2.0
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS-IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */


jimport("org.ccil.cowan.tagsoup.Parser");
jimport("org.ccil.cowan.tagsoup.PYXWriter");
jimport("java.io.StringReader");
jimport("java.io.StringWriter");
jimport("org.xml.sax.InputSource");

import("etherpad.collab.ace.easysync2.{Changeset,AttribPool}");
import("etherpad.collab.ace.contentcollector.makeContentCollector");
import("etherpad.collab.collab_server");

function setPadHTML(pad, html) {
  var atext = htmlToAText(html, pad.pool());
  collab_server.setPadAText(pad, atext);
}

function _html2pyx(html) {
  var p = new Parser();
  var w = new StringWriter();
  var h = new PYXWriter(w);
  p.setContentHandler(h);
  var s = new InputSource();
  s.setCharacterStream(new StringReader(html));
  p.parse(s);
  return w.toString().replace(/\r\n|\r|\n/g, '\n');
}

// added the following logic - JHOLMES
var indentation = '';

function trimLeadingWhitespace(str) {
  while (str.substring(0, 1) == ' ') {
    str = str.substring(1);
  }
  return str;
}

// for every .5in of text-indent, insert a tab character, which will be coverted to 7 non-breaking spaces
// if the text-indent CSS rule's value is not specified in inches, simply insert a single tab character
function indentToNbsp(val) {
  if (val.indexOf('in') > -1) {
    var theValueIndex = val.indexOf('in');
    var theValue = val.substring(0, theValueIndex);
    var theNum   = new Number(theValue);
    var spaces   = Math.round(theNum * 2);
    for (y = 0; y < spaces; y++) {
      indentation = indentation + '\t';
    }
  }
  else {
    indentation = indentation + '\t';
  }
}

function parseForIndentation(inlineStyle) {
  var cssDeclaration;
  var cssProperty;
  var cssValue;
  var cssRules = inlineStyle.split(';');
  for (x = 0; x < cssRules.length; x++) {
    if (cssRules[x] != '') {
      cssDeclaration = cssRules[x].split(':');
      cssProperty    = trimLeadingWhitespace(cssDeclaration[0]);
      cssValue       = trimLeadingWhitespace(cssDeclaration[1]);
      if (cssProperty == 'text-indent') {
        indentToNbsp(cssValue);
      }
    }
  }
  return indentation;
}
// end of logic added here - JHOLMES

function _htmlBody2js(html) {
  var pyx = _html2pyx(html);
  var plines = pyx.split("\n");

  function pyxUnescape(s) {
    return s.replace(/\\t/g, '\t').replace(/\\/g, '\\');
  }
  var inAttrs = false;

  var nodeStack = [];
  var topNode = {};

  var bodyNode = {name:"body"};

  plines.forEach(function(pline) {
    var t = pline.charAt(0);
    var v = pline.substring(1);
    if (inAttrs && t != 'A') {
      inAttrs = false;
    }
    if (t == '?') { /* ignore */ }
    else if (t == '(') {
      var newNode = {name: v};
      if (v.toLowerCase() == "body") {
        bodyNode = newNode;
      }
      topNode.children = (topNode.children || []);
      topNode.children.push(newNode);
      nodeStack.push(topNode);
      topNode = newNode;
      inAttrs = true;
    }
    else if (t == 'A') {
      var spaceIndex = v.indexOf(' ');
      var key = v.substring(0, spaceIndex);
      var value = pyxUnescape(v.substring(spaceIndex+1));
      topNode.attrs = (topNode.attrs || {});
      topNode.attrs['$'+key] = value;
      // added the following logic here - JHOLMES
      if ((key.toLowerCase() == "style") && (value.indexOf('text-indent') > -1)) {
        indentation = parseForIndentation(value);
      }
      // end of logic added here - JHOLMES
    }
    else if (t == '-') {
      if (v == "\\n") {
        v = '\n';
      }
      else {
        v = pyxUnescape(v);
      }
      if (v) {
        topNode.children = (topNode.children || []);
        if (topNode.children.length > 0 &&
            ((typeof topNode.children[topNode.children.length-1]) == "string")) {
          // coallesce
          // prepend the indentation value here then clear the valuable - JHOLMES
          topNode.children.push(indentation + topNode.children.pop() + v);
        }
        else {
          // prepend the indentation value here then clear the valuable - JHOLMES
          topNode.children.push(indentation + v);
        }
        indentation = "";
      }
    }
    else if (t == ')') {
      topNode = nodeStack.pop();
    }
  });

  return bodyNode;
}

function _trimDomNode(n) {
  function isWhitespace(str) {
    return /^\s*$/.test(str);
  }
  function trimBeginningOrEnd(n, endNotBeginning) {
    var cc = n.children;
    var backwards = endNotBeginning;
    if (cc) {
      var i = (backwards ? cc.length-1 : 0);
      var done = false;
      var hitActualText = false;
      while (! done) {
        if (! (backwards ? (i >= 0) : (i < cc.length-1))) {
          done = true;
        }
        else {
          var c = cc[i];
          if ((typeof c) == "string") {
            if (! isWhitespace(c)) {
              // actual text
              hitActualText = true;
              break;
            }
            else {
              // whitespace
              cc[i] = '';
            }
          }
          else {
            // recurse
            if (trimBeginningOrEnd(cc[i], endNotBeginning)) {
              hitActualText = true;
              break;
            }
          }
          i += (backwards ? -1 : 1);
        }
      }
      n.children = n.children.filter(function(x) { return !!x; });
      return hitActualText;
    }
    return false;
  }
  trimBeginningOrEnd(n, false);
  trimBeginningOrEnd(n, true);
}

//convert "$Attribute" in sever side into "Attribute" in client
function _safeAttributeName(name){
  name = name || "";
  if("$" == name[0]){
    name = name.substr(1);
  }
  return name;
}

function htmlToAText(html, apool) {
  var body = _htmlBody2js(html);
  _trimDomNode(body);

  var dom = {
    isNodeText: function(n) {
      return (typeof n) == "string";
    },
    nodeTagName: function(n) {
      return ((typeof n) == "object") && n.name;
    },
    nodeValue: function(n) {
      return String(n);
    },
    nodeNumChildren: function(n) {
      return (((typeof n) == "object") && n.children && n.children.length) || 0;
    },
    nodeChild: function(n, i) {
      return (((typeof n) == "object") && n.children && n.children[i]) || null;
    },
    nodeProp: function(n, p) {
      if("className" == p){
        p = "class";
      }
      return (((typeof n) == "object") && n.attrs && (n.attrs[p] || n.attrs["$" + p])) || null;
    },
    nodeAttr: function(n, a) {
      return (((typeof n) == "object") && n.attrs && (n.attrs[a] || n.attrs["$" + a])) || null;
    },
    nodeAttributes : function(n){
      var attribs = {}, na = n.attrs;
      if(na){
        for(var name in na){
          attribs[_safeAttributeName(name)] = na[name];
        }
      }
      return attribs;
    },
    optNodeInnerHTML: function(n) {
      return null;
    }
  }

  var cc = makeContentCollector(true, null, apool, dom);
  for(var i=0; i<dom.nodeNumChildren(body); i++) {
    var n = dom.nodeChild(body, i);
    cc.collectContent(n);
  }
  cc.notifyNextNode(null);
  var ccData = cc.finish();

  var textLines = ccData.lines;
  var attLines = ccData.lineAttribs;
  for(var i=0;i<textLines.length;i++) {
    var txt = textLines[i];
    if (txt == " " || txt == "\xa0") {
      // space or nbsp all alone on a line, remove
      textLines[i] = "";
      attLines[i] = "";
    }
  }

  var text = textLines.join('\n')+'\n';
  var attribs = _joinLineAttribs(attLines);
  var atext = Changeset.makeAText(text, attribs);

  return atext;
}

function _joinLineAttribs(lineAttribs) {
  var assem = Changeset.smartOpAssembler();

  var newline = Changeset.newOp('+');
  newline.chars = 1;
  newline.lines = 1;

  lineAttribs.forEach(function(aline) {
    var iter = Changeset.opIterator(aline);
    while (iter.hasNext()) {
      assem.append(iter.next());
    }
    assem.append(newline);
  });

  return assem.toString();
}
