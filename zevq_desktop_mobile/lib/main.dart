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
      title: 'Zevq AI',
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
  String _report = 'Awaiting local audit.';

  void _runAudit() {
    setState(() {
      _report = _bridge.auditSource('fn main(){ let x = 1 / (z - 5); }');
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Zevq AI Air-Gapped Console', style: TextStyle(fontSize: 32, fontWeight: FontWeight.w700)),
            const SizedBox(height: 16),
            ElevatedButton(style: ElevatedButton.styleFrom(backgroundColor: const Color(0xFFFF6B35)), onPressed: _runAudit, child: const Text('Run Local Rust Audit')),
            const SizedBox(height: 24),
            Container(width: double.infinity, padding: const EdgeInsets.all(20), decoration: BoxDecoration(color: const Color(0xFFFF94B4), borderRadius: BorderRadius.circular(20)), child: Text(_report, style: const TextStyle(color: Color(0xFF4B5563)))),
          ],
        ),
      ),
    );
  }
}
