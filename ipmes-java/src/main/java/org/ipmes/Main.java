package org.ipmes;

import java.io.BufferedReader;
import java.io.FileReader;
import java.util.ArrayList;
import java.util.stream.Collectors;

import io.siddhi.core.SiddhiAppRuntime;
import io.siddhi.core.SiddhiManager;
import io.siddhi.core.stream.input.InputHandler;

public class Main {
    public static void main(String[] args) throws Exception {
        // parse data
        String ttpPrefix = args[0];
        boolean useRegex = false;
        String orelsFile;
        if (ttpPrefix.endsWith("regex")) {
            useRegex = true;
            orelsFile = ttpPrefix.substring(0, ttpPrefix.length() - 5) + "oRels.json";
        } else {
            orelsFile = ttpPrefix + "_oRels.json";
        }
        PatternGraph pattern = PatternGraph
                .parse(new FileReader(ttpPrefix + "_node.json"), new FileReader(ttpPrefix + "_edge.json")).get();
        DependencyGraph dep = DependencyGraph.parse(new FileReader(orelsFile)).get();

        // Decomposition
        Decomposition d = new Decomposition(dep, pattern);
        ArrayList<TCQuery> tcQueries = d.decompose();
        TCSiddhiAppGenerator gen = new TCSiddhiAppGenerator(pattern, dep, tcQueries);
        gen.setUseRegex(useRegex);

        // Generate CEP app and runtime
        String appStr = gen.generate();
        System.out.println(appStr);
        SiddhiManager siddhiManager = new SiddhiManager();
        SiddhiAppRuntime runtime = siddhiManager.createSiddhiAppRuntime(appStr);

        Join join = new Join(dep, pattern);
        for (TCQuery q : tcQueries) {
            runtime.addCallback(
                    String.format("TC%dOutput", q.getId()),
                    new TCQueryOutputCallback(q, pattern, join));
        }

        String inputFile = args[1];
        BufferedReader inputReader = new BufferedReader(new FileReader(inputFile));
        String line = inputReader.readLine();
        InputHandler inputHandler = runtime.getInputHandler("InputStream");
        runtime.start();

        EventSorter sorter = new EventSorter(tcQueries);
        ArrayList<EventEdge> timeBuffer = new ArrayList<>();
        while (line != null) {
            EventEdge event = new EventEdge(line);
            if (!timeBuffer.isEmpty()) {
                if (event.timestamp.equals(timeBuffer.get(0).timestamp)) {
                    timeBuffer.add(event);
                } else {
                    ArrayList<EventEdge> sorted = sorter.rearrange(timeBuffer);
                    for (EventEdge edge : sorted)
                        inputHandler.send(edge.toEventData());
                    timeBuffer.clear();
                }
            } else {
                timeBuffer.add(event);
            }
            line = inputReader.readLine();
        }

        System.out.println("Match Results:");
        ArrayList<ArrayList<MatchEdge>> results = join.extractAnswer();
        for (ArrayList<MatchEdge> result : results) {
            System.out.print("[");
            System.out.print(
                result.stream()
                    .map(edge -> edge.getDataId().toString())
                    .collect(Collectors.joining(", "))
            );
            System.out.println("]");
        }

        runtime.shutdown();
        siddhiManager.shutdown();
    }
}