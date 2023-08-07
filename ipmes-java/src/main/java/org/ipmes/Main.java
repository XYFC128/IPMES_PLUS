package org.ipmes;

import java.io.BufferedReader;
import java.io.FileReader;
import java.util.ArrayList;
import java.util.stream.Collectors;

import io.siddhi.core.SiddhiAppRuntime;
import io.siddhi.core.SiddhiManager;
import io.siddhi.core.stream.input.InputHandler;
import org.ipmes.decomposition.TCQGenerator;
import org.ipmes.decomposition.TCQuery;
import org.ipmes.decomposition.TCQueryRelation;
import org.ipmes.match.MatchEdge;
import org.ipmes.pattern.DependencyGraph;
import org.ipmes.pattern.PatternGraph;
import org.ipmes.siddhi.TCQueryOutputCallback;
import org.ipmes.siddhi.TCSiddhiAppGenerator;

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
        TCQGenerator d = new TCQGenerator(dep, pattern);
        ArrayList<TCQuery> tcQueries = d.decompose();
        ArrayList<TCQueryRelation>[] TCQRelation = d.getTCQRelation();
        TCSiddhiAppGenerator gen = new TCSiddhiAppGenerator(pattern, dep, tcQueries);
        gen.setUseRegex(useRegex);

        // Generate CEP app and runtime
        String appStr = gen.generate();
        System.out.println(appStr);
        SiddhiManager siddhiManager = new SiddhiManager();
        SiddhiAppRuntime runtime = siddhiManager.createSiddhiAppRuntime(appStr);
        Join join = new Join(dep, pattern, TCQRelation);
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

        EventSorter sorter = new EventSorter(tcQueries, useRegex);
        ArrayList<EventEdge> timeBuffer = new ArrayList<>();
        while (line != null) {
            EventEdge event = new EventEdge(line);
            if (!timeBuffer.isEmpty() && !event.timestamp.equals(timeBuffer.get(0).timestamp)) {
                ArrayList<Object[]> sorted = sorter.rearrangeToEventData(timeBuffer);
                for (Object[] data : sorted)
                    inputHandler.send(data);
                timeBuffer.clear();
            }
            timeBuffer.add(event);
            line = inputReader.readLine();
        }
        if (!timeBuffer.isEmpty()) {
            ArrayList<Object[]> sorted = sorter.rearrangeToEventData(timeBuffer);
            for (Object[] data : sorted)
                inputHandler.send(data);
        }

        System.out.println("Match Results:");
        ArrayList<ArrayList<MatchEdge>> results = join.extractAnswer();
        for (ArrayList<MatchEdge> result : results) {
            System.out.print("[");
            System.out.print(
                    result.stream()
                            .map(edge -> edge.getDataId().toString())
                            .collect(Collectors.joining(",")));
            System.out.println("]");
        }

        runtime.shutdown();
        siddhiManager.shutdown();
    }
}