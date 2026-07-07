import 'package:flutter/material.dart';

import 'rust_bridge.dart';

void main() {
  runApp(const ZevqShell());
}

class ZevqShell extends StatelessWidget {
  const ZevqShell({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'Zevq AI Ratings',
      theme: ThemeData(
        scaffoldBackgroundColor: const Color(0xFFFFFFFF),
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFFFF6B35)),
        textTheme: Theme.of(context).textTheme.apply(bodyColor: const Color(0xFF1F2937), displayColor: const Color(0xFF1F2937)),
      ),
      home: const ZevqHome(),
    );
  }
}

class ZevqHome extends StatefulWidget {
  const ZevqHome({super.key});

  @override
  State<ZevqHome> createState() => _ZevqHomeState();
}

class _ZevqHomeState extends State<ZevqHome> {
  final _bridge = ZevqRustBridge();
  final _profileController = TextEditingController(text: '''{
  "name": "Confident AI",
  "domain": "ai_evaluation",
  "claimed_accuracy": 0.94,
  "eval_pass_rate": 0.87,
  "adversarial_resilience": 0.71,
  "data_governance": 0.68,
  "incidents_last_90_days": 2,
  "p95_latency_ms": 420.0,
  "has_human_override": false
}''');
  String _report = 'Paste a startup evidence JSON profile, then run the local Rust rating engine.';

  void _runAssessment() {
    setState(() {
      _report = _bridge.assessStartup(_profileController.text);
    });
  }

  @override
  void dispose() {
    _profileController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text('Zevq AI Startup Ratings', style: TextStyle(fontSize: 34, fontWeight: FontWeight.w800)),
              const SizedBox(height: 8),
              const Text('Mathematical safety, reliability, and governance scoring for AI startups.', style: TextStyle(color: Color(0xFF4B5563), fontSize: 16)),
              const SizedBox(height: 24),
              TextField(
                controller: _profileController,
                minLines: 8,
                maxLines: 12,
                decoration: InputDecoration(
                  filled: true,
                  fillColor: const Color(0xFFF9FAFB),
                  focusedBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(20), borderSide: const BorderSide(color: Color(0xFFFF6B35), width: 2)),
                  border: OutlineInputBorder(borderRadius: BorderRadius.circular(20)),
                  labelText: 'Startup evidence JSON profile',
                ),
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  ElevatedButton(style: ElevatedButton.styleFrom(backgroundColor: const Color(0xFFFF6B35), foregroundColor: Colors.white), onPressed: _runAssessment, child: const Text('Run Mathematical Rating')),
                  const SizedBox(width: 16),
                  const Text('Offline Rust engine via FFI', style: TextStyle(color: Color(0xFF4B5563))),
                ],
              ),
              const SizedBox(height: 24),
              Expanded(
                child: Container(
                  width: double.infinity,
                  padding: const EdgeInsets.all(20),
                  decoration: BoxDecoration(color: const Color(0xFFFF94B4), borderRadius: BorderRadius.circular(20)),
                  child: SingleChildScrollView(child: Text(_report, style: const TextStyle(color: Color(0xFF4B5563), fontFamily: 'monospace'))),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
