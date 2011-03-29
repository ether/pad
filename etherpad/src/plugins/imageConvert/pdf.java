/*
import java.io.*;
import java.util.*;
import java.nio.*;
import java.nio.channels.*;
import java.awt.image.*;
import java.awt.*;
import javax.imageio.*;

import org.pdfbox.pdmodel.*;

public class pdf{
    public static void main(String [] args) throws Exception {
        String file = args[0];
        PDDocument document = PDDocument.load(file);
        java.util.List page = document.getDocumentCatalog().getPage(0);

        java.util.List pages = document.getDocumentCatalog().getAllPages();
        for(int i=0;i<pages.size();++i){
	    PDPage page = (PDPage)pages.get( i );
	    BufferedImage image = page.convertToImage();
	    ImageIO.write(image,"png",new File("pdf"+i+".png"));
        }
    }
}

*/

import java.lang.ProcessBuilder;
import java.lang.Process;
import java.nio.*;
import java.io.*;


public class pdf{

    public static float[] getSize(String filename)
        throws Exception {
        String myFilename = filename + "[0]";
        
        ProcessBuilder pb = new ProcessBuilder("identify","-format", "%[fx:w]\n%[fx:h]",myFilename);
        Process p = pb.start();
        InputStream is = p.getInputStream();
        BufferedReader d = new BufferedReader(new InputStreamReader(is));
        String w = d.readLine();
        String h = d.readLine();
        p.waitFor();
        float res[] =  {Float.parseFloat(w), Float.parseFloat(h)};
        return res;
    }
    
    public static void convert(String filename, int page, String out, int offset[], int size[], int pixelSize[]) 
        throws Exception {
        String myFilename = filename + "["+(page-1)+"]";
        String myScale = "" + pixelSize[0]+"x" + pixelSize[1];
        String myCrop = "" + size[0]+"x" + size[1] + "+" + offset[0] + "+" + offset[1];
        ProcessBuilder pb = new ProcessBuilder("convert", "-crop", myCrop, "-scale", myScale, myFilename, out);
        Process p = pb.start();
        p.waitFor();
    }
    
    public static void main(String arg[])
        throws Exception { 
        System.out.println(pdf.getSize("test.pdf")[0]);
        System.out.println(pdf.getSize("test.pdf")[1]);
        int offset[] =  {400, 500};
        int size[] =  {500, 500};
        int pSize[] =  {200,200};
        pdf.convert("test.pdf", 1, "test.png", offset, size, pSize);
   }
    
}