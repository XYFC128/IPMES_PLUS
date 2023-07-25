package org.ipmes;

import java.io.FileReader;

public class Main {
    public static void main(String[] args) throws Exception {
        PatternGraph g = PatternGraph.parse(new FileReader(args[0]), new FileReader(args[1])).get();
        System.out.println(g.getNodes());
        System.out.println(g.getEdges());
//        // Create Siddhi Manager
//        SiddhiManager siddhiManager = new SiddhiManager();
//
//        //Siddhi Application
//        String siddhiApp = "" +
//                "define stream InputStream (id int, msg string); " +
//                "" +
//                "@info(name = 'query1') " +
//                "from InputStream#window.expressionBatch(\"first.id == last.id\") " +
//                "select * " +
//                "insert into OutputStream;";
//
//        //Generate runtime
//        SiddhiAppRuntime siddhiAppRuntime = siddhiManager.createSiddhiAppRuntime(siddhiApp);
//
//        //Adding callback to retrieve output events from stream
//        siddhiAppRuntime.addCallback("OutputStream", new StreamCallback() {
//            @Override
//            public void receive(Event[] events) {
//                if (events.length < 2)
//                    return;
//                System.out.println("Events: ");
//                for (Event e : events) {
//                    System.out.println(e.toString());
//                }
//                //To convert and print event as a map
//                //EventPrinter.print(toMap(events));
//            }
//        });
//
//        //Get InputHandler to push events into Siddhi
//        InputHandler inputHandler = siddhiAppRuntime.getInputHandler("InputStream");
//
//        //Start processing
//        siddhiAppRuntime.start();
//
//        //Sending events to Siddhi
//        inputHandler.send(new Object[]{1, "Hello"});
//        inputHandler.send(new Object[]{1, "there"});
//        inputHandler.send(new Object[]{2, "Hi"});
//        inputHandler.send(new Object[]{2, "you"});
//        inputHandler.send(new Object[]{3, "aaa"});
//        inputHandler.send(new Object[]{4, "bbb"});
//        Thread.sleep(500);
//
//        //Shutdown runtime
//        siddhiAppRuntime.shutdown();
//
//        //Shutdown Siddhi Manager
//        siddhiManager.shutdown();
    }
}